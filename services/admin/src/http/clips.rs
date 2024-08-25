use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use twitch::models::GameId;

use crate::{prelude::*, views::Views};

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

impl Views {
    pub fn clips(
        &self,
        db: &DbConn,
        game_id: &GameId,
        total_count: usize,
        clips: &[models::clip::Clip],
        query: &models::clip::ShowParams,
    ) -> Result<Html<String>> {
        trace!("Rendering clips for game {game_id}");

        let game = db::game::select_by_id(db, game_id)?;

        self.handlebars
            .render(
                "clips",
                &json!({
                    "parent": "base_xl",
                    "game": game,
                    "total_count": total_count,
                    "query": query,
                    "clips": clips,
                }),
            )
            .map(Html)
            .map_err(From::from)
    }
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Query(mut query): Query<models::clip::ShowParams>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;

    query.pagination.default_page_size_if_none(&db)?;

    trace!("Listing clips for game {game_id}");
    let (total_count, clips) = db::clip::list(&db, &game_id, &query)?;

    s.views.clips(&db, &game_id, total_count, &clips, &query)
}
