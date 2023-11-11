pub mod fetch_new_game_clips;
pub mod map_clips_to_groups;

use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;

use crate::prelude::*;

#[derive(Clone)]
pub struct Jobs {
    pub scheduler: JobScheduler,
    pub fetch_new_game_clips: uuid::Uuid,
    pub map_clips_to_groups: uuid::Uuid,
}

pub async fn schedule_all(
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
    })
}
