use axum::{
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;

use crate::prelude::*;

pub async fn show(State(s): State<g::HttpState>) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.settings(&db)
}

#[derive(Deserialize)]
pub struct EditSettings {
    fetch_new_game_clips_cron: Option<String>,
    recorded_at_least_hours_ago: Option<i64>,
}

pub async fn edit(
    State(s): State<g::HttpState>,
    Form(settings): Form<EditSettings>,
) -> Result<Redirect> {
    let EditSettings {
        fetch_new_game_clips_cron,
        recorded_at_least_hours_ago,
    } = settings;

    let db = s.db.lock().await;
    if let Some(cron) = fetch_new_game_clips_cron {
        db::setting::update(&db, "fetch_new_game_clips_cron", &cron)?;
    }
    if let Some(hours) = recorded_at_least_hours_ago {
        db::setting::update(
            &db,
            "recorded_at_least_hours_ago",
            &hours.to_string(),
        )?;
    }

    Ok(Redirect::to("/settings"))
}
