use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
};
use std::collections::HashMap;

use crate::prelude::*;

pub async fn search(
    State(s): State<g::HttpState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>> {
    let query = params
        .get("q")
        .ok_or_else(|| AppError::bad_request("Missing query param 'q'"))?;

    let games = s.twitch.search_for_game(query).await?;

    s.views.search_game(&games)
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.game(&db, &game_id)
}

pub async fn add(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    let Some(game) = s.twitch.get_game(game_id.clone()).await? else {
        return Err(AppError::bad_request(format!(
            "Game with id {game_id} not found"
        )));
    };

    let db = s.db.lock().await;
    db::game::insert(&db, &game)?;

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

pub async fn delete(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    let db = s.db.lock().await;
    db::game::delete(&db, &game_id)?;

    Ok(Redirect::to("/"))
}

pub async fn pause(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), true).await?;

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

pub async fn resume(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), false).await?;

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

async fn set_is_paused(
    s: &g::HttpState,
    game_id: twitch::models::GameId,
    is_paused: bool,
) -> Result<()> {
    let db = s.db.lock().await;
    db::game::set_is_paused(&db, &game_id, is_paused)?;

    Ok(())
}
