use serde::{Deserialize, Serialize};
use twitch_api2::types::{CategoryId, TwitchCategory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Game {
    pub id: GameId,
    pub name: String,
    /// Note that this _might_ contain a placeholder for the size.
    ///
    /// `https://static-cdn.jtvnw.net/ttv-boxart/{game_id}-{width}x{height}.jpg`
    pub box_art_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct GameId(String);

pub fn into_standard_box_art_size(box_art_url: impl AsRef<str>) -> String {
    box_art_url
        .as_ref()
        .replace("{width}", &(52 * 4).to_string())
        .replace("{height}", &(70 * 4).to_string())
}

impl From<TwitchCategory> for Game {
    fn from(c: TwitchCategory) -> Self {
        Self {
            id: c.id.into(),
            name: c.name,
            box_art_url: c.box_art_url,
        }
    }
}

impl From<String> for GameId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for GameId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<GameId> for String {
    fn from(g: GameId) -> Self {
        g.0
    }
}

impl From<CategoryId> for GameId {
    fn from(c: CategoryId) -> Self {
        Self(c.into_string())
    }
}

impl From<GameId> for CategoryId {
    fn from(g: GameId) -> Self {
        g.into_string().into()
    }
}

impl GameId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "sqlite")]
impl rusqlite::types::FromSql for GameId {
    fn column_result(
        value: rusqlite::types::ValueRef<'_>,
    ) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|s| Self(s.to_string()))
    }
}

#[cfg(feature = "sqlite")]
impl rusqlite::types::ToSql for GameId {
    fn to_sql(
        &self,
    ) -> std::result::Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error>
    {
        Ok(rusqlite::types::ToSqlOutput::from(self.0.clone()))
    }
}
