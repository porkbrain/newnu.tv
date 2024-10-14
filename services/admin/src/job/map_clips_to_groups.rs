//! Some groups have query params which are used to map clips to them.
//! Here, we select the clips which match these criteria and then map them.

use anyhow::anyhow;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    models::{
        clip::{GroupParams, ShowParams},
        group::Group,
    },
    prelude::*,
};

pub async fn schedule(
    scheduler: &mut JobScheduler,
    db: DbLock,
) -> AnyResult<uuid::Uuid> {
    let map_clips_to_groups_cron = {
        let db = db.lock().await;
        db::setting::map_clips_to_groups_cron(&db)?
    };

    let map_clips_to_groups = scheduler
        .add(Job::new_async(map_clips_to_groups_cron, move |_, _| {
            debug!("Triggering map_clips_to_groups");

            let db = Arc::clone(&db);

            Box::pin(async move {
                let res = once_for_all(Arc::clone(&db)).await;
                if let Err(e) = res {
                    error!("Cannot trigger map_clips_to_groups: {e}");
                }
            })
        })?)
        .await?;

    let next_tick = scheduler
        .next_tick_for_job(map_clips_to_groups)
        .await?
        .ok_or_else(|| {
            anyhow!("Cannot find next tick for map_clips_to_groups")
        })?;
    info!("Next tick for map_clips_to_groups: {next_tick}");

    Ok(map_clips_to_groups)
}

pub async fn once(db: DbLock, group_id: &GroupId) -> Result<()> {
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
        db::group::select_all_with_metadata(&db)?
    };

    once_(db, groups).await
}

async fn once_(db: DbLock, groups: Vec<Group>) -> Result<()> {
    for Group {
        name,
        metadata_json,
        id: gid,
        ..
    } in groups
    {
        let Some(meta) = metadata_json else {
            warn!("Group '{name}' has no metadata_json");
            continue;
        };
        let query = match serde_json::from_value::<GroupParams>(meta) {
            Ok(query) => query,
            Err(e) => {
                warn!("Group '{name}' has invalid metadata structure: {e}");
                continue;
            }
        };
        let GroupParams { game_id, filter } = query;

        let mut db = db.lock().await;
        let (_, clips) = db::clip::list(
            &db,
            &game_id,
            &ShowParams {
                filter,
                ..Default::default()
            },
        )?;

        let tx = db.transaction()?;
        for clip in clips {
            let manual = false;
            db::group::map_clip(&tx, &gid, &clip.id, manual)?;
        }
        tx.commit()?;
    }

    Ok(())
}
