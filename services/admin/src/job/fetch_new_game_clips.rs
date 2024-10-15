use anyhow::anyhow;
use db::{clip::InsertClipStatement, game::GameWithLatestClipRecordedAt};
use rand::Rng;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, error::TryRecvError, Sender, UnboundedSender},
    time::{sleep, sleep_until, Instant},
};
use tokio_cron_scheduler::{Job, JobScheduler};
use twitch::twitch_api2::types::Timestamp;

use crate::prelude::*;

/// Good default will have some hours because that allows for views to grow on
/// clips before they are fetched.
const HOW_LONG_UNTIL_CLIP_HAS_REASONABLE_VIEWS: Duration =
    Duration::from_secs(8 * 3600);

const JOB_NAME: &str = "fetch_new_game_clips";

#[derive(Debug, Default)]
pub(crate) struct Conf {
    /// Only clips newer than this will be fetched.
    pub(crate) recorded_at_most_ago: Option<chrono::Duration>,
    /// Only clips older than this will be fetched.
    pub(crate) recorded_at_least_ago: Option<chrono::Duration>,
}

/// Like a breath-first search, we start with a seed where we have one element
/// for each game and new elements are added to the end of the queue based on
/// result of pagination.
struct QueueElement {
    /// We can feed this type to `twitch::Client::get_clips_paginated`.
    request: twitch::models::GetClipsRequest,
    /// If we get 409 from Twitch, we retry a few times.
    /// If this reaches 3, we give up.
    retry_count: u8,
    /// We pass handle back to the channel which orchestrates how often we
    /// perform new requests.
    ///
    /// One last queue element is destroyed, then what remains of the whole
    /// iteration of this job is storing in db.
    /// That's because the `QueueElement`` channel receiver will be dropped as
    /// no more senders exist.
    fetch_clips: Sender<QueueElement>,
}

pub(crate) async fn schedule(
    scheduler: &mut JobScheduler,
    db: DbLock,
    tc: Arc<twitch::Client>,
) -> AnyResult<uuid::Uuid> {
    let (fetch_new_game_clips_cron, recorded_at_least_ago) = {
        let db = db.lock().await;
        let new_clips_cron = db::setting::fetch_new_game_clips_cron(&db)?;
        let at_least_ago = db::setting::recorded_at_least_hours_ago(&db)?;

        (new_clips_cron, Some(chrono::Duration::hours(at_least_ago)))
    };

    debug!("Scheduling {JOB_NAME} with cron '{fetch_new_game_clips_cron}'");

    let fetch_new_game_clips = scheduler
        .add(Job::new_async(fetch_new_game_clips_cron, move |_, _| {
            debug!("Triggering {JOB_NAME}");

            let db = Arc::clone(&db);
            let tc = Arc::clone(&tc);

            Box::pin(async move {
                let res = once_for_all(
                    Arc::clone(&db),
                    Arc::clone(&tc),
                    Conf {
                        recorded_at_most_ago: None, // last clip
                        recorded_at_least_ago,
                    },
                )
                .await;
                if let Err(e) = res {
                    error!("Cannot trigger {JOB_NAME}: {e}");
                }
            })
        })?)
        .await?;

    let next_tick = scheduler
        .next_tick_for_job(fetch_new_game_clips)
        .await?
        .ok_or_else(|| anyhow!("Cannot find next tick for {JOB_NAME}"))?;
    info!("Next tick for {JOB_NAME}: {next_tick}");

    Ok(fetch_new_game_clips)
}

pub(crate) async fn once(
    db: DbLock,
    tc: Arc<twitch::Client>,
    conf: Conf,
    game_id: twitch::models::GameId,
) -> Result<()> {
    let recorded_at = {
        let db = db.lock().await;
        db::game::select_latest_clip_recorded_at(&db, &game_id)?
    };

    once_(
        db,
        tc,
        conf,
        vec![GameWithLatestClipRecordedAt {
            game_id,
            latest_clip_recorded_at: recorded_at,
        }],
    )
    .await
}

async fn once_for_all(
    db: DbLock,
    tc: Arc<twitch::Client>,
    conf: Conf,
) -> Result<()> {
    let game_ids = {
        let db = db.lock().await;
        db::game::select_all_active_that_have_last_clip_older_than(
            &db,
            conf.recorded_at_least_ago.unwrap_or(
                chrono::Duration::from_std(
                    HOW_LONG_UNTIL_CLIP_HAS_REASONABLE_VIEWS,
                )
                .unwrap(),
            ),
        )?
    };

    once_(db, tc, conf, game_ids).await
}

#[instrument(skip(db, tc))]
async fn once_(
    db: DbLock,
    tc: Arc<twitch::Client>,
    conf: Conf,
    game_ids: Vec<GameWithLatestClipRecordedAt>,
) -> Result<()> {
    if game_ids.is_empty() {
        debug!("No games to fetch clips for, skipping job");
        return Ok(());
    }

    info!("Fetching new clips");

    let store_clips = spawn_channel_to_store_clips(db);
    // there'll be (at most) 1 request per game as pagination requires a request
    // to be finished before new one can be sent
    let (fetch_clips, mut next_clip_request) =
        mpsc::channel::<QueueElement>(game_ids.len());

    for GameWithLatestClipRecordedAt {
        game_id,
        latest_clip_recorded_at,
    } in game_ids
    {
        if let Some(recorded_at) = latest_clip_recorded_at {
            debug!("Game {game_id} has latest clip recorded at {recorded_at}");
        } else {
            debug!("Game {game_id} has no clips yet");
        }

        let request = twitch::models::GetClipsRequest::builder()
            .game_id(Some(game_id.into()))
            .first(100)
            .started_at(
                conf.recorded_at_most_ago
                    .map(|at| chrono::Utc::now() - at) // user precedence
                    .or(latest_clip_recorded_at) // else use latest (if set)
                    .or_else(|| {
                        // otherwise default to 2 days ago for new games
                        Some(chrono::Utc::now() - chrono::Duration::days(2))
                    })
                    .and_then(|t| Timestamp::new(t.to_rfc3339()).ok()),
            )
            .ended_at(
                Some(
                    chrono::Utc::now()
                        - conf
                            .recorded_at_least_ago
                            .unwrap_or(chrono::Duration::days(1)),
                )
                .and_then(|t| Timestamp::new(t.to_rfc3339()).ok()),
            )
            .build();

        let fetch_clips_to_send = fetch_clips.clone();
        fetch_clips
            .send(QueueElement {
                request,
                retry_count: 0,
                fetch_clips: fetch_clips_to_send,
            })
            .await
            .expect("Channel not closed bcs both ends in scope");
    }

    // IMPORTANT!
    // All handles which determine whether new messages can be received were
    // already created in the loop above.
    //
    // Those handles will be passed into new tasks.
    // We drop this handle to ensure that the loop below will end, otherwise
    // it'd be infinite.
    drop(fetch_clips);

    //
    // main job task start
    //

    // TODO: move this as a channel impl to twitch crate to have one global
    // rate limiter
    'outer: loop {
        // rate limit is 800/minute, we round that down to 10/s
        const MAX_REQUESTS_PER_SECOND: usize = 10;

        let next_second = sleep_until(Instant::now() + Duration::from_secs(1));

        for _ in 0..MAX_REQUESTS_PER_SECOND {
            let Some(el) = next_clip_request.recv().await else {
                break 'outer;
            };

            spawn_task_to_fetch_clip(Arc::clone(&tc), store_clips.clone(), el);
        }

        // if it was more than 1s, this immediately resolves
        next_second.await;
    }

    //
    // main job task end
    //

    Ok(())
}

/// Send clips down this channel to get them persisted in db.
///
/// This channel will keep running as long as the main job task scope lives.
/// Then the handle to the sender is dropped.
fn spawn_channel_to_store_clips(
    db: DbLock,
) -> UnboundedSender<twitch::models::Clip> {
    let (store_clips, mut next_clip) =
        mpsc::unbounded_channel::<twitch::models::Clip>();

    tokio::spawn(async move {
        const BUFFER_CAP: usize = 128;

        let mut buffer = Vec::with_capacity(BUFFER_CAP);

        let store =
            |db: &mut DbConn, buffer: &mut Vec<twitch::models::Clip>| {
                let tx = db.transaction()?;
                let mut stmt = InsertClipStatement::new(&tx)?;

                for clip in buffer.iter() {
                    stmt.execute(clip)?;
                }

                drop(stmt);
                tx.commit()?;

                buffer.clear();

                AnyResult::<()>::Ok(())
            };

        let res = loop {
            match next_clip.try_recv() {
                Ok(clip) => {
                    buffer.push(clip);

                    if buffer.len() >= BUFFER_CAP {
                        if let Err(e) =
                            store(&mut *db.lock().await, &mut buffer)
                        {
                            break Err(e);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    if buffer.is_empty() {
                        tokio::time::sleep({
                            // arbitrary, since it's an unbounded channel we
                            // don't hold anything back
                            const SLEEP_DURATION: Duration =
                                Duration::from_millis(50);
                            SLEEP_DURATION
                        })
                        .await;
                    } else if let Err(e) =
                        store(&mut *db.lock().await, &mut buffer)
                    {
                        break Err(e);
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    if let Err(e) = store(&mut *db.lock().await, &mut buffer) {
                        break Err(e);
                    }
                    break Ok(());
                }
            }
        };

        if let Err(e) = res {
            error!("Failed to store {} clips in db: {e}", buffer.len());
        }
    });

    store_clips
}

/// Spawns a new task which will fetch clips for a given request.
///
/// If the request is paginated, it will spawn a new task for the next page.
/// The difference between the new paginated request is only the cursor.
///
/// Any fetched clips will be sent down the channel to store them in db.
fn spawn_task_to_fetch_clip(
    tc: Arc<twitch::Client>,
    channel_to_store_clips: UnboundedSender<twitch::models::Clip>,
    el: QueueElement,
) {
    let QueueElement {
        request,
        retry_count,
        fetch_clips,
    } = el;

    tokio::spawn(async move {
        match tc.get_clips_paginated(request.clone()).await {
            Ok((clips, cursor)) => {
                let clips_len = clips.len();
                let game_id =
                    request.game_id.expect("Game ID is always present");

                for clip in clips {
                    if channel_to_store_clips.send(clip).is_err() {
                        error!("Failed to send clip to store, channel closed");
                    }
                }

                if let Some(next_request) = cursor {
                    debug!("Fetched {clips_len} more clips for {game_id}");

                    if fetch_clips
                        .clone()
                        .send(QueueElement {
                            request: next_request,
                            retry_count: 0,
                            fetch_clips,
                        })
                        .await
                        .is_err()
                    {
                        error!("Main channel closed, cannot send next page");
                    }
                } else {
                    debug!("Fetched last {clips_len} clips for {game_id}",);
                }
            }
            Err(e) if e.to_string().contains("409") => {
                if retry_count < 3 {
                    // TODO: move retry logic to twitch crate
                    let delay_ms = rand::thread_rng().gen_range(1_000..8_000);

                    warn!("Rate limited, retrying after {delay_ms}ms");
                    sleep(Duration::from_millis(delay_ms)).await;

                    if fetch_clips
                        .clone()
                        .send(QueueElement {
                            request,
                            retry_count: retry_count + 1,
                            fetch_clips,
                        })
                        .await
                        .is_err()
                    {
                        error!("Main channel closed, cannot resend page");
                    }
                } else {
                    error!(
                        "Rate limited and retry count exceeded (game {})",
                        request.game_id.expect("Game ID is always present")
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to get clips for game {}: {e}",
                    request.game_id.expect("Game ID is always present")
                );
            }
        }
    });
}
