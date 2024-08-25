use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_json: Option<serde_json::Value>,
    pub zindex: i64,
    pub archived: bool,
    pub download_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub struct GroupId(usize);

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl rusqlite::types::FromSql for GroupId {
    fn column_result(
        value: rusqlite::types::ValueRef<'_>,
    ) -> rusqlite::types::FromSqlResult<Self> {
        value.as_i64().map(|s| Self(s as usize))
    }
}

impl rusqlite::types::ToSql for GroupId {
    fn to_sql(
        &self,
    ) -> std::result::Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error>
    {
        Ok(rusqlite::types::ToSqlOutput::from(self.0 as i64))
    }
}
