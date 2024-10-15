pub(crate) use crate::conf::Conf;
pub(crate) use crate::db;
pub(crate) use crate::error::AppError;
pub(crate) use crate::g;
pub(crate) use crate::models::{self, clip::ClipId, group::GroupId};
pub(crate) use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
pub(crate) use axum::{extract::State, Json};
pub(crate) use std::result::Result as StdResult;
pub(crate) use tracing::instrument;
pub(crate) use tracing::{debug, error, info, trace, warn};

pub(crate) type DbConn = rusqlite::Connection;
pub(crate) type DbLock = std::sync::Arc<tokio::sync::Mutex<DbConn>>;
pub(crate) type Result<T = (), E = AppError> = std::result::Result<T, E>;
