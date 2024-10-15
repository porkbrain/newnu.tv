use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
};
use serde_json::json;
use std::collections::HashMap;
use twitch::models::GameId;

use crate::{prelude::*, views::Views};

impl Views {
    pub(crate) fn search_game(
        &self,
        games: &[twitch::models::Game],
    ) -> Result<Html<String>> {
        self.handlebars
            .render(
                "search_game",
                &json!({
                    "parent": "base",
                    "games": games
                }),
            )
            .map(Html)
            .map_err(From::from)
    }
}

pub(crate) async fn search(
    State(s): State<g::HttpState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>> {
    let query = params
        .get("q")
        .ok_or_else(|| AppError::bad_request("Missing query param 'q'"))?;

    let games = s.twitch.search_for_game(query).await?;

    s.views.search_game(&games)
}

impl Views {
    /// Pull all necessary data to render game info from db.
    pub(crate) fn game(
        &self,
        db: &DbConn,
        game: &GameId,
    ) -> Result<Html<String>> {
        let game = db::game::select_by_id(db, game)?;

        self.handlebars
            .render("game", &json!({ "parent": "base", "game": game }))
            .map(Html)
            .map_err(From::from)
    }
}

pub(crate) async fn show(
    State(s): State<g::HttpState>,
    Path(game_id): Path<GameId>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.game(&db, &game_id)
}

pub(crate) async fn add(
    State(s): State<g::HttpState>,
    Path(game_id): Path<GameId>,
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

pub(crate) async fn delete(
    State(s): State<g::HttpState>,
    Path(game_id): Path<GameId>,
) -> Result<Redirect> {
    let db = s.db.lock().await;
    db::game::delete(&db, &game_id)?;

    Ok(Redirect::to("/"))
}

pub(crate) async fn pause(
    State(s): State<g::HttpState>,
    Path(game_id): Path<GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), true).await?;

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

pub(crate) async fn resume(
    State(s): State<g::HttpState>,
    Path(game_id): Path<GameId>,
) -> Result<Redirect> {
    set_is_paused(&s, game_id.clone(), false).await?;

    Ok(Redirect::to(&format!("/games/{game_id}")))
}

async fn set_is_paused(
    s: &g::HttpState,
    game_id: GameId,
    is_paused: bool,
) -> Result<()> {
    let db = s.db.lock().await;
    db::game::set_is_paused(&db, &game_id, is_paused)?;

    Ok(())
}
