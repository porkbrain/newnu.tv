use crate::prelude::*;
use anyhow::Context;
use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub(crate) struct Conf {
    /// For example 0.0.0.0:8080
    pub(crate) socket_addr: SocketAddr,
    pub(crate) sqlite_db_path: PathBuf,
    pub(crate) worker_addr: SocketAddr,
    pub(crate) twitch_client_id: String,
    pub(crate) twitch_secret: String,
}

impl Conf {
    pub(crate) fn from_env() -> AnyResult<Self> {
        info!("Loading config from environment");

        let socket_addr = env::var("SOCKET_ADDR").context("SOCKET_ADDR")?;
        debug!("SOCKET_ADDR: {socket_addr}");

        let worker_addr = env::var("WORKER_ADDR").context("WORKER_ADDR")?;
        debug!("WORKER_ADDR: {worker_addr}");

        let sqlite_db_path =
            env::var("SQLITE_DB_PATH").context("SQLITE_DB_PATH")?;
        debug!("SQLITE_DB_PATH: {sqlite_db_path}");

        let twitch_client_id =
            env::var("TWITCH_CLIENT_ID").context("TWITCH_CLIENT_ID")?;
        debug!(
            "TWITCH_CLIENT_ID: {}{}",
            &twitch_client_id[0..3],
            "*".repeat(twitch_client_id.len() - 3)
        );

        let twitch_secret =
            env::var("TWITCH_SECRET").context("TWITCH_SECRET")?;
        debug!(
            "TWITCH_SECRET: {}{}",
            &twitch_secret[0..3],
            "*".repeat(twitch_secret.len() - 3)
        );

        Ok(Self {
            socket_addr: socket_addr.parse()?,
            worker_addr: worker_addr.parse()?,
            sqlite_db_path: sqlite_db_path.into(),
            twitch_client_id,
            twitch_secret,
        })
    }

    pub(crate) fn db_path(&self) -> &Path {
        self.sqlite_db_path.as_ref()
    }

    /// The worker does not have to be running for this to succeed.
    pub(crate) async fn connect_lazy_worker_client(
        &self,
    ) -> AnyResult<worker::Client> {
        let dst = format!("http://{}", self.worker_addr);
        let conn = tonic::transport::Endpoint::new(dst)?.connect_lazy();
        Ok(worker::Client::new(conn))
    }

    pub(crate) async fn construct_twitch_client(
        &self,
    ) -> AnyResult<twitch::Client> {
        twitch::Client::new(
            self.twitch_client_id.as_str(),
            self.twitch_secret.as_str(),
        )
        .await
    }
}
