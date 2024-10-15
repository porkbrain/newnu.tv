//! Some groups have query params which are used to map clips to them.
//! Here, we select the clips which match these criteria and then map them.

use anyhow::anyhow;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    models::{
        clip::{Filter, Pagination, ShowParams},
        group::Group,
    },
    prelude::*,
};

const JOB_NAME: &str = "download_clips";

pub(crate) async fn schedule(
    scheduler: &mut JobScheduler,
    db: DbLock,
) -> AnyResult<uuid::Uuid> {
    let download_clips_cron = {
        let db = db.lock().await;
        db::setting::download_clips_cron(&db)?
    };

    debug!("Scheduling {JOB_NAME} with cron '{download_clips_cron}'");

    let download_clips = scheduler
        .add(Job::new_async(download_clips_cron, move |_, _| {
            debug!("Triggering {JOB_NAME}");

            let db = Arc::clone(&db);

            Box::pin(async move {
                let res = once_for_all(Arc::clone(&db)).await;
                if let Err(e) = res {
                    error!("Cannot trigger {JOB_NAME}: {e}");
                }
            })
        })?)
        .await?;

    let next_tick = scheduler
        .next_tick_for_job(download_clips)
        .await?
        .ok_or_else(|| anyhow!("Cannot find next tick for {JOB_NAME}"))?;
    info!("Next tick for {JOB_NAME}: {next_tick}");

    Ok(download_clips)
}

pub(crate) async fn once(db: DbLock, group_id: &GroupId) -> Result<()> {
    let group = {
        let db = db.lock().await;
        db::group::select_by_id(&db, group_id)?
    };

    once_(db, vec![group]).await
}

/// Selects all groups with metadata defined.
/// Skips invalid metadata gracefully.
async fn once_for_all(db: DbLock) -> Result<()> {
    let groups = {
        let db = db.lock().await;
        let all_groups = db::group::select_all(&db)?;
        // lazier to filter here than in the sql query
        all_groups
            .into_iter()
            .filter(|g| g.download_enabled)
            .collect()
    };

    once_(db, groups).await
}

async fn once_(db: DbLock, groups: Vec<Group>) -> Result<()> {
    // TODO: implement the following algorithm:
    //      let before_check = now()
    //      let x = setting::last_checked_downloaded
    //      worker::clips_downloaded_since(x) (fs::metadata().modified())
    //      set clips is_downloaded = true to all ^
    //      settings::set(last_checked_downloaded, before_check)

    for Group { id: group_id, .. } in groups {
        let (total, clips) = {
            let db = db.lock().await;
            db::clip::select_by_group_id(
                &db,
                &group_id,
                &ShowParams {
                    pagination: Pagination {
                        // download at most 1000 undownloaded clips per group,
                        // pretty arbitrary so no need for a setting
                        //
                        // in the next iteration, the downloaded clips will have
                        // been marked as such, so we can download others
                        page_size: Some(1000),
                        ..Default::default()
                    },
                    filter: Filter {
                        // is_downloaded: Some(false),
                        ..Default::default()
                    },
                },
            )?
        };

        info!(
            "Downloading {clips_len}/{total} clips for group {group_id}",
            clips_len = clips.len(),
        );

        // TODO: collect all clips into a hashmap by their id
    }

    // TODO: send a request to the worker to download the clips:
    //      worker.download_clips(clips.into_iter()
    //      .map(|c| rpc::Clip {id: c.id, url: c.url}))

    Ok(())
}
