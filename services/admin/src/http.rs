/// endpoints for clips management
mod clips;
/// endpoints which ease development
mod dev;
/// endpoints for twitch category management
mod game;
/// endpoints for group (of clips) management
mod group;
/// homepage
mod home;
/// endpoints for global settings
mod settings;

use crate::prelude::*;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::json;

pub(crate) async fn start(g: g::HttpState) -> AnyResult<()> {
    let socket_addr = g.conf.socket_addr;

    let routes = routes().with_state(g);

    info!("http://{socket_addr}");
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, routes).await.unwrap();

    Ok(())
}

fn routes() -> Router<g::HttpState> {
    async fn handler_404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Invalid path",
            })),
        )
    }

    // since data is being sent with <form> which only supports GET and POST,
    // we instead append the method verb to the path (with the exception of GET)
    //
    // this is a trade-off between ~dogma~ conventions and no-js life
    // i'll take the latter any day of the week
    let app = Router::new()
        .route("/", get(home::page))
        .route("/search/games", get(game::search))
        .route("/games/:game_id", get(game::show))
        .route("/games/:game_id/post", post(game::add))
        .route("/games/:game_id/delete", post(game::delete))
        .route("/games/:game_id/pause/post", post(game::pause))
        .route("/games/:game_id/pause/delete", post(game::resume))
        .route("/games/:game_id/clips", get(clips::show))
        .route(
            "/games/:game_id/clips/groups/post",
            post(group::from_clips_query),
        )
        .route(
            "/games/:game_id/clips/fetch/post",
            post(clips::trigger_fetch),
        )
        .route("/groups/post", post(group::create))
        .route("/groups/:group_id", get(group::show))
        .route("/groups/:group_id/archive/post", post(group::archive))
        .route(
            "/groups/:group_id/download/auto/post",
            post(group::enable_download),
        )
        .route(
            "/groups/:group_id/download/auto/delete",
            post(group::disable_download),
        )
        .route(
            "/groups/:group_id/download/post",
            post(group::trigger_download),
        )
        .route("/groups/:group_id/map/post", post(group::trigger_mapping))
        .route("/settings", get(settings::show))
        .route("/settings/put", post(settings::edit))
        .route("/dev/reset/post", post(dev::reset))
        .fallback(handler_404);

    debug!("Http routes constructed");
    app
}
