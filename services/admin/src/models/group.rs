use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Group {
    pub(crate) id: GroupId,
    pub(crate) name: String,
    pub(crate) color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata_json: Option<serde_json::Value>,
    pub(crate) zindex: i64,
    pub(crate) archived: bool,
    pub(crate) download_enabled: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub(crate) struct GroupId(usize);

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
