pub(crate) mod download_clips;
pub(crate) mod fetch_new_game_clips;
pub(crate) mod map_clips_to_groups;

use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;

use crate::prelude::*;

#[derive(Clone)]
#[allow(dead_code)] // TODO: we will allow restarting of jobs later
pub(crate) struct Jobs {
    pub(crate) scheduler: JobScheduler,
    pub(crate) fetch_new_game_clips: uuid::Uuid,
    pub(crate) map_clips_to_groups: uuid::Uuid,
    pub(crate) download_clips: uuid::Uuid,
}

pub(crate) async fn schedule_all(
    db: DbLock,
    tc: Arc<twitch::Client>,
) -> AnyResult<Jobs> {
    let mut scheduler = JobScheduler::new().await?;

    let fetch_new_game_clips_job = fetch_new_game_clips::schedule(
        &mut scheduler,
        Arc::clone(&db),
        Arc::clone(&tc),
    )
    .await?;

    let map_clips_to_groups_job =
        map_clips_to_groups::schedule(&mut scheduler, Arc::clone(&db)).await?;

    let download_clips_job =
        download_clips::schedule(&mut scheduler, Arc::clone(&db)).await?;

    let shed = scheduler.clone();
    tokio::spawn(async move {
        if let Err(e) = shed.start().await {
            error!("Error in scheduler: {e}");
        }
    });

    Ok(Jobs {
        scheduler,
        fetch_new_game_clips: fetch_new_game_clips_job,
        map_clips_to_groups: map_clips_to_groups_job,
        download_clips: download_clips_job,
    })
}
