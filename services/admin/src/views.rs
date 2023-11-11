use crate::prelude::*;
use axum::response::Html;
use handlebars::Handlebars;
use serde_json::json;
use std::sync::Arc;
use twitch::models::GameId;

#[derive(Clone)]
pub struct Views {
    handlebars: Arc<Handlebars<'static>>,
}

impl Views {
    pub fn new() -> AnyResult<Self> {
        let mut h = Handlebars::new();
        h.set_strict_mode(false);

        helpers::register_all(&mut h);

        // contains the head imports
        h.register_template_string("base", include_str!("views/base.hbs"))?;
        h.register_template_string(
            "base_xl",
            include_str!("views/base_xl.hbs"),
        )?;

        h.register_template_string(
            "homepage",
            include_str!("views/homepage.hbs"),
        )?;

        h.register_template_string(
            "search_game",
            include_str!("views/search_game.hbs"),
        )?;

        h.register_template_string("game", include_str!("views/game.hbs"))?;

        h.register_template_string(
            "settings",
            include_str!("views/settings.hbs"),
        )?;

        h.register_template_string("clips", include_str!("views/clips.hbs"))?;

        h.register_template_string("group", include_str!("views/group.hbs"))?;

        Ok(Self {
            handlebars: Arc::new(h),
        })
    }

    /// Pull all necessary data to render homepage from db.
    pub fn homepage(&self, db: &DbConn) -> Result<Html<String>> {
        let games: Vec<_> = db::game::select_all(db)?;

        self.handlebars
            .render("homepage", &json!({ "parent": "base", "games": games }))
            .map(Html)
            .map_err(From::from)
    }

    pub fn search_game(
        &self,
        games: &[twitch::models::Game],
    ) -> Result<Html<String>> {
        self.handlebars
            .render(
                "search_game",
                &json!({
                    "parent": "base",
                    "games": games
                }),
            )
            .map(Html)
            .map_err(From::from)
    }

    /// Pull all necessary data to render game info from db.
    pub fn game(&self, db: &DbConn, game: &GameId) -> Result<Html<String>> {
        let game = db::game::select_by_id(db, game)?;

        self.handlebars
            .render("game", &json!({ "parent": "base", "game": game }))
            .map(Html)
            .map_err(From::from)
    }

    /// View global settings.
    pub fn settings(&self, db: &DbConn) -> Result<Html<String>> {
        let fetch_new_game_clips_cron =
            db::setting::fetch_new_game_clips_cron(db)?;
        let recorded_at_least_hours_ago =
            db::setting::recorded_at_least_hours_ago(db)?;
        let map_clips_to_groups_cron =
            db::setting::map_clips_to_groups_cron(db)?;
        let default_page_size = db::setting::default_page_size(db)?;

        self.handlebars
            .render(
                "settings",
                &json!(
                    {
                        "parent": "base",
                        "settings": {
                            "fetch_new_game_clips_cron": fetch_new_game_clips_cron,
                            "recorded_at_least_hours_ago": recorded_at_least_hours_ago,
                            "map_clips_to_groups_cron": map_clips_to_groups_cron,
                            "default_page_size": default_page_size,
                        }
                    }
                ),
            )
            .map(Html)
            .map_err(From::from)
    }

    pub fn clips(
        &self,
        db: &DbConn,
        game_id: &GameId,
        total_count: usize,
        clips: &[models::clip::Clip],
        query: &models::clip::ShowParams,
    ) -> Result<Html<String>> {
        trace!("Rendering clips for game {game_id}");

        let game = db::game::select_by_id(db, game_id)?;

        self.handlebars
            .render(
                "clips",
                &json!({
                    "parent": "base_xl",
                    "game": game,
                    "total_count": total_count,
                    "query": query,
                    "clips": clips,
                }),
            )
            .map(Html)
            .map_err(From::from)
    }

    pub fn group(
        &self,
        group: &models::group::Group,
        total_count: usize,
        clips: &[models::clip::Clip],
        query: &models::clip::Pagination,
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

mod helpers {
    use handlebars::{handlebars_helper, Handlebars};
    use serde_json::Value;

    pub fn register_all(handlebars: &mut Handlebars<'_>) {
        handlebars.register_helper("div", Box::new(div));
        handlebars.register_helper("add", Box::new(add));
        handlebars.register_helper("equals", Box::new(equals));
        handlebars.register_helper("not", Box::new(not));
        handlebars.register_helper("contains", Box::new(contains));
        handlebars.register_helper("empty", Box::new(empty));
    }

    handlebars_helper!(div: |a: usize, b: usize| a / b);
    handlebars_helper!(add: |a: usize, b: usize| a + b);
    handlebars_helper!(equals: |a: Value, b: Value| a == b);
    handlebars_helper!(not: |a: Value| match a {
        Value::Bool(b) => !b,
        Value::Null => true,
        _ => false,
    });
    handlebars_helper!(contains: |a: Value, v: Value| match a {
        Value::Array(a) => a.contains(&v),
        Value::Null => false,
        _ => panic!("contains helper expects array and string"),
    });
    handlebars_helper!(empty: |v: Value| match v {
        Value::Array(a) => a.is_empty(),
        Value::String(s) => s.is_empty(),
        Value::Object(m) => m.is_empty(),
        Value::Null => true,
        _ => false,
    });
}
