use crate::job::Jobs;
use crate::prelude::*;
use crate::views::Views;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct HttpState {
    pub(crate) db: Arc<Mutex<DbConn>>,
    pub(crate) conf: Arc<Conf>,
    pub(crate) views: Views,
    pub(crate) _worker: Arc<Mutex<worker::Client>>,
    pub(crate) twitch: Arc<twitch::Client>,
    pub(crate) _jobs: Jobs,
}

pub(crate) fn empty_string_is_none<'de, D, T: FromStr>(
    deserializer: D,
) -> StdResult<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(T::from_str(&s).ok())
    }
}

pub(crate) fn csv_string_is_vec<'de, D>(
    deserializer: D,
) -> StdResult<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Either {
        String(String),
        Vec(Vec<String>),
    }
    match Either::deserialize(deserializer)? {
        Either::String(s) => Ok(s
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()),
        Either::Vec(v) => Ok(v),
    }
}

pub(crate) fn string_is_usize<'de, D>(
    deserializer: D,
) -> StdResult<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Either {
        String(String),
        Usize(usize),
    }
    match Either::deserialize(deserializer)? {
        Either::String(s) if s.is_empty() => {
            Err(serde::de::Error::missing_field("empty string"))
        }
        Either::String(s) => s.parse().map_err(serde::de::Error::custom),
        Either::Usize(u) => Ok(u),
    }
}

pub(crate) fn string_is_option_usize<'de, D>(
    deserializer: D,
) -> StdResult<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        String(String),
        Usize(usize),
    }
    match Either::deserialize(deserializer)? {
        Either::String(s) if s.is_empty() => Ok(None),
        Either::String(s) => {
            Ok(Some(s.parse().map_err(serde::de::Error::custom)?))
        }
        Either::Usize(u) => Ok(Some(u)),
    }
}

pub(crate) fn string_is_bool<'de, D>(
    deserializer: D,
) -> StdResult<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        String(String),
        Bool(bool),
    }

    match Either::deserialize(deserializer)? {
        Either::String(s) => s.parse().map_err(serde::de::Error::custom),
        Either::Bool(b) => Ok(b),
    }
}
