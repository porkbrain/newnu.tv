use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use twitch::models::GameId;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ClipId(String);

#[derive(Debug, Serialize)]
pub struct Clip {
    pub broadcaster_id: String,
    pub broadcaster_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub creator_name: String,
    pub duration: Duration,
    pub game_id: String,
    pub id: ClipId,
    pub lang: String,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub thumbnail_url: String,
    pub title: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub url: String,
    pub view_count: usize,
    /// Whether the worker has downloaded the clip.
    /// Useful to know for example for exports - if not all clips are
    /// downloaded, then the export will take longer.
    ///
    /// Also, we don't instruct the worker to download clips that are already
    /// downloaded.
    /// We prefer to store state in admin.
    pub is_downloaded: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ShowParams {
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(flatten)]
    pub filter: Filter,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GroupParams {
    pub game_id: GameId,
    #[serde(flatten)]
    pub filter: Filter,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Pagination {
    #[serde(default)]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub page_size: Option<usize>,
    #[serde(default)]
    #[serde(deserialize_with = "g::string_is_usize")]
    pub page_offset: usize,
    #[serde(default)]
    #[serde(deserialize_with = "g::string_is_bool")]
    pub sort_direction_asc: bool,
    #[serde(default)]
    pub sort_by: ShowSortBy,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Filter {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub broadcaster_name: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub title_like: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(deserialize_with = "g::csv_string_is_vec")]
    pub langs: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::string_is_option_usize")]
    pub view_count_max: Option<usize>,
    #[serde(default)]
    #[serde(deserialize_with = "g::string_is_usize")]
    pub view_count_min: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub min_recorded_at: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub max_recorded_at: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "g::empty_string_is_none")]
    pub is_downloaded: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShowSortBy {
    RecordedAt,
    #[default]
    ViewCount,
}

impl Pagination {
    pub fn page_size_or_default(&self, db: &DbConn) -> Result<usize> {
        match self.page_size {
            Some(0) | None => Ok(db::setting::default_page_size(db)?),
            Some(ps) => Ok(ps),
        }
    }

    pub fn default_page_size_if_none(&mut self, db: &DbConn) -> Result<()> {
        if matches!(self.page_size, Some(0) | None) {
            self.page_size = Some(db::setting::default_page_size(db)?)
        }

        Ok(())
    }
}

impl From<ShowSortBy> for &'static str {
    fn from(s: ShowSortBy) -> Self {
        match s {
            ShowSortBy::RecordedAt => "recorded_at",
            ShowSortBy::ViewCount => "view_count",
        }
    }
}

impl From<(GameId, ShowParams)> for GroupParams {
    fn from((game_id, p): (GameId, ShowParams)) -> Self {
        Self {
            game_id,
            filter: p.filter,
        }
    }
}

impl std::fmt::Display for ClipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl rusqlite::types::FromSql for ClipId {
    fn column_result(
        value: rusqlite::types::ValueRef<'_>,
    ) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|s| Self(s.to_string()))
    }
}

impl rusqlite::types::ToSql for ClipId {
    fn to_sql(
        &self,
    ) -> std::result::Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error>
    {
        Ok(rusqlite::types::ToSqlOutput::from(self.0.clone()))
    }
}

impl PartialEq<&str> for ClipId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl From<&str> for ClipId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
