use crate::job::Jobs;
use crate::prelude::*;
use crate::views::Views;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct HttpState {
    pub db: Arc<Mutex<DbConn>>,
    pub conf: Arc<Conf>,
    pub views: Views,
    pub worker: Arc<Mutex<worker::Client>>,
    pub twitch: Arc<twitch::Client>,
    pub _jobs: Jobs,
}

pub fn empty_string_is_none<'de, D, T: FromStr>(
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

pub fn csv_string_is_vec<'de, D>(
    deserializer: D,
) -> StdResult<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

pub fn string_is_usize<'de, D>(deserializer: D) -> StdResult<usize, D::Error>
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

pub fn string_is_option_usize<'de, D>(
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

pub fn string_is_bool<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
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
