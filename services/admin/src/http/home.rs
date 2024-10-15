use crate::{prelude::*, views::Views};
use axum::response::Html;
use serde_json::json;

impl Views {
    /// Pull all necessary data to render homepage from db.
    pub(crate) fn homepage(&self, db: &DbConn) -> Result<Html<String>> {
        let games = db::game::select_all(db)?;
        let groups = db::group::select_all(db)?;

        self.handlebars
            .render(
                "homepage",
                &json!({ "parent": "base", "games": games, "groups": groups }),
            )
            .map(Html)
            .map_err(From::from)
    }
}

pub(crate) async fn page(
    State(s): State<g::HttpState>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;
    s.views.homepage(&db)
}
