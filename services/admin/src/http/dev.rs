use crate::prelude::*;
use axum::response::Redirect;

pub(crate) async fn reset(State(s): State<g::HttpState>) -> Result<Redirect> {
    let mut db = s.db.lock().await;
    db::down(&mut db)?;
    db::up(&mut db)?;

    Ok(Redirect::to("/"))
}
