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
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use serde_json::json;
use std::net::SocketAddr;

pub async fn start(g: g::HttpState) -> AnyResult<()> {
    let http_addr = g.conf.http_addr;

    let routes = routes().with_state(g);

    info!("http://{http_addr}");
    axum::Server::bind(&http_addr)
        .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

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
        .route("/settings", get(settings::show))
        .route("/settings/put", post(settings::edit))
        .route("/dev/reset/post", post(dev::reset))
        .fallback(handler_404);

    debug!("Http routes constructed");
    app
}
