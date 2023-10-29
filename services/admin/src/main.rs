/// Global config structure loaded from .env file
mod conf;
/// Database schema, models and helpers
mod db;
/// Error conversions into http
mod error;
/// Global state
mod g;
/// Sets up http server with router
mod http;
/// Global imports of ubiquitous types
mod prelude;
/// Http templates with handlebars
mod views;

use crate::prelude::*;
use crate::views::Views;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenvy::from_filename(".env.admin").ok();
    pretty_env_logger::try_init_timed().ok();

    info!("web admin starting");

    let conf = Conf::from_env()?;
    let worker = conf.connect_lazy_worker_client().await?;
    let twitch = conf.construct_twitch_client().await?;
    let mut db = conf.open_db()?;
    db::up(&mut db)?;

    let g = g::HttpState {
        conf: Arc::new(conf),
        db: Arc::new(Mutex::new(db)),
        worker: Arc::new(Mutex::new(worker)),
        views: Views::new()?,
        twitch: Arc::new(twitch),
    };

    http::start(g).await?;

    Ok(())
}