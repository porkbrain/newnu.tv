use axum::{
    extract::{Path, Query},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::{models::clip::GroupParams, prelude::*, views::Views};

/// For convenience we use the same struct as the clip listing.
/// This in turn means we get the params from query rather than body.
pub async fn from_clips_query(
    State(s): State<g::HttpState>,
    Path(game_id): Path<twitch::models::GameId>,
    Query(query): Query<models::clip::Filter>,
) -> Result<Redirect> {
    let db = s.db.lock().await;

    // get current datetime in UTC and append it to the game name
    let now = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        .to_string()
        .replace("T", " ");
    let now = now.split('+').next().unwrap();
    let game_name = db::game::select_by_id(&db, &game_id)?.name;
    let group_name = format!("{game_name} {now}");

    let color = generate_random_color();

    let group_id = db::group::insert(&db, &group_name, &color)?;
    db::group::update_metadata_json(
        &db,
        &group_id,
        serde_json::to_value(GroupParams {
            game_id,
            filter: query,
        })?,
    )?;

    drop(db);

    crate::job::map_clips_to_groups::once(Arc::clone(&s.db), &group_id).await?;

    Ok(Redirect::to(&format!("/groups/{group_id}")))
}

#[derive(Deserialize)]
pub struct NewGroup {
    name: String,
    /// If not provided, then one is generated.
    color: Option<String>,
}
pub async fn create(
    State(s): State<g::HttpState>,
    Form(NewGroup { name, color }): Form<NewGroup>,
) -> Result<Redirect> {
    let color = color.unwrap_or_else(generate_random_color);

    let db = s.db.lock().await;
    let group_id = db::group::insert(&db, &name, &color)?;

    Ok(Redirect::to(&format!("/groups/{group_id}")))
}

impl Views {
    pub fn group(
        &self,
        group: &models::group::Group,
        total_count: usize,
        clips: &[models::clip::Clip],
        query: &models::clip::ShowParams,
    ) -> Result<Html<String>> {
        trace!("Rendering clips for group {}", group.id);

        self.handlebars
            .render(
                "group",
                &json!({
                    "parent": "base_xl",
                    "group": group,
                    "total_count": total_count,
                    "clips": clips,
                    "query": query,
                }),
            )
            .map(Html)
            .map_err(From::from)
    }
}

pub async fn show(
    State(s): State<g::HttpState>,
    Path(group_id): Path<GroupId>,
    Query(mut query): Query<models::clip::ShowParams>,
) -> Result<Html<String>> {
    let db = s.db.lock().await;

    query.pagination.default_page_size_if_none(&db)?;

    let group = db::group::select_by_id(&db, &group_id)?;
    let (total_count, clips) =
        db::clip::select_by_group_id(&db, &group_id, &query)?;

    s.views.group(&group, total_count, &clips, &query)
}

pub async fn archive(
    State(s): State<g::HttpState>,
    Path(group_id): Path<GroupId>,
) -> Result<Redirect> {
    let db = s.db.lock().await;

    db::group::archive(&db, &group_id)?;

    Ok(Redirect::to("/"))
}

pub async fn enable_download(
    State(s): State<g::HttpState>,
    Path(group_id): Path<GroupId>,
) -> Result<Redirect> {
    let db = s.db.lock().await;

    db::group::set_auto_download(&db, &group_id, true)?;

    Ok(Redirect::to(&format!("/groups/{group_id}")))
}

pub async fn disable_download(
    State(s): State<g::HttpState>,
    Path(group_id): Path<GroupId>,
) -> Result<Redirect> {
    let db = s.db.lock().await;

    db::group::set_auto_download(&db, &group_id, true)?;

    Ok(Redirect::to(&format!("/groups/{group_id}")))
}

pub async fn trigger_download(
    State(s): State<g::HttpState>,
    Path(group_id): Path<GroupId>,
) -> Result<Redirect> {
    crate::job::download_clips::once(Arc::clone(&s.db), &group_id).await?;

    Ok(Redirect::to(&format!("/groups/{group_id}")))
}

fn generate_random_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("#{:x}", rng.gen::<u32>() / 256)
}
