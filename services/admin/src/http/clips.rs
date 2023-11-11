use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::prelude::*;

#[derive(Deserialize, Debug)]
pub struct TriggerFetchClipsJob {
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub recorded_at_most_hours_ago: Option<usize>,
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub recorded_at_least_hours_ago: Option<usize>,
}
pub async fn trigger_fetch(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Form(TriggerFetchClipsJob {
        recorded_at_most_hours_ago: at_most,
        recorded_at_least_hours_ago: at_least,
    }): Form<TriggerFetchClipsJob>,
) -> Result<Redirect> {
    let at_most = at_most.unwrap_or_default();
    let at_least = at_least.unwrap_or_default();
    if at_most != 0 && at_most <= at_least {
        return Err(AppError::bad_request(
            "Recorded 'at most' must be greater than to 'at least'",
        ));
    }

    info!(
        "Triggering fetch new game clips job for game {game_id} in the \
        last {at_most} hours, at least {at_least} hours ago"
    );

    tokio::spawn(crate::job::fetch_new_game_clips::once(
        Arc::clone(&s.db),
        Arc::clone(&s.twitch),
        crate::job::fetch_new_game_clips::Conf {
            recorded_at_most_ago: if at_most == 0 {
                None
            } else {
                Some(chrono::Duration::hours(at_most as i64))
            },
            recorded_at_least_ago: if at_least == 0 {
                None
            } else {
                Some(chrono::Duration::hours(at_least as i64))
            },
        },
        game_id.clone(),
    ));

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Query(mut query): Query<models::clip::ShowParams>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;

    if query.pagination.page_size.is_none() {
        query.pagination.page_size = Some(db::setting::default_page_size(&db)?);
    }

    trace!("Listing clips for game {game_id}");
    let (total_count, clips) = db::clip::list(&db, &game_id, &query)?;

    s.views.clips(&db, &game_id, total_count, &clips, &query)
}
