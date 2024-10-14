use crate::prelude::*;
use anyhow::Context;
use std::{env, net::SocketAddr};

pub(crate) struct Conf {
    /// For example 0.0.0.0:8080
    pub(crate) rpc_addr: SocketAddr,
}

impl Conf {
    pub(crate) fn from_env() -> AnyResult<Self> {
        info!("Loading config from environment");

        let rpc_addr = env::var("RPC_ADDR").context("RPC_ADDR")?;
        debug!("RPC_ADDR: {rpc_addr}");

        Ok(Self {
            rpc_addr: rpc_addr.parse()?,
        })
    }
}
