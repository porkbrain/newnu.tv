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
/// Cron job scheduling.
mod job;
/// Global structs and their implementation
mod models;
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
    let worker = Arc::new(Mutex::new(conf.connect_lazy_worker_client().await?));
    let tc = Arc::new(conf.construct_twitch_client().await?);
    let mut db = db::open(conf.db_path())?;
    db::up(&mut db)?;
    let db = Arc::new(Mutex::new(db));

    let jobs = job::schedule_all(Arc::clone(&db), Arc::clone(&tc)).await?;

    let g = g::HttpState {
        conf: Arc::new(conf),
        db,
        _worker: worker,
        views: Views::new()?,
        twitch: tc,
        _jobs: jobs,
    };

    http::start(g).await?;

    Ok(())
}
