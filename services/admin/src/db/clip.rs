use anyhow::Context;
use itertools::Itertools;
use rusqlite::{named_params, ToSql};
use std::{rc::Rc, time::Duration};
use twitch::models::GameId;

use crate::models::clip::{Clip, Filter, ShowParams};
use crate::prelude::*;

/// Bind clips to it and execute all inserts at once.
pub(crate) struct InsertClipStatement<'a>(rusqlite::Statement<'a>);

impl<'a> InsertClipStatement<'a> {
    pub(crate) fn new(db: &'a rusqlite::Connection) -> Result<Self> {
        // replace will update view count
        let stmt = db.prepare(
            "
            INSERT OR REPLACE INTO
            clips(
                id,
                broadcaster_id,
                broadcaster_name,
                creator_name,
                recorded_at,
                duration,
                title,
                thumbnail_url,
                url,
                view_count,
                lang,
                game_id
            )
            VALUES (
                :id,
                :broadcaster_id,
                :broadcaster_name,
                :creator_name,
                :recorded_at,
                :duration,
                :title,
                :thumbnail_url,
                :url,
                :view_count,
                :lang,
                :game_id
            );",
        )?;

        Ok(Self(stmt))
    }

    pub(crate) fn execute(
        &mut self,
        clip: &twitch::models::Clip,
    ) -> Result<()> {
        self.0
            .execute(rusqlite::named_params! {
                ":id": clip.id,
                ":broadcaster_id": clip.broadcaster_id,
                ":broadcaster_name": clip.broadcaster_name,
                ":creator_name": clip.creator_name,
                ":recorded_at": clip.recorded_at,
                ":duration": (clip.duration.as_secs() as i64),
                ":title": clip.title,
                ":thumbnail_url": clip.thumbnail_url,
                ":url": clip.url,
                ":view_count": (clip.view_count as i64),
                ":lang": clip.lang,
                ":game_id": clip.game_id,
            })
            .map_err(|e| anyhow!("Cannot bind clip: {e}"))?;

        Ok(())
    }
}

pub(crate) fn list(
    db: &DbConn,
    game_id: &GameId,
    ShowParams { pagination, filter }: &ShowParams,
) -> Result<(usize, Vec<Clip>)> {
    let sql_filter = sql_filter();

    let total_count_sql =
        format!("SELECT COUNT(*) FROM clips WHERE {sql_filter}");
    let mut params = Params::new(filter);
    let total_count: i64 = {
        let mut p = params.as_named();
        p.push((":game_id", game_id));
        db.prepare(&total_count_sql)?
            .query_row(p.as_slice(), |row| row.get(0))
            .context("clip::list total count")?
    };

    let page_size = pagination.page_size_or_default(db)?;
    params.add_pagination(page_size, pagination.page_offset);
    let sort_by: &str = From::from(pagination.sort_by);
    let sort_direction = if pagination.sort_direction_asc {
        "ASC"
    } else {
        "DESC"
    };

    let paginated_sql = format!(
        "SELECT
            broadcaster_id,
            broadcaster_name,
            created_at,
            creator_name,
            duration,
            game_id,
            id,
            lang,
            recorded_at,
            thumbnail_url,
            title,
            updated_at,
            url,
            view_count,
            is_downloaded
        FROM clips
        WHERE {sql_filter}
        ORDER BY {sort_by} {sort_direction}
        LIMIT :page_size
        OFFSET :page_offset",
    );
    let clips = {
        let mut p = params.as_named();
        p.push((":game_id", game_id));
        db.prepare_cached(&paginated_sql)?
            .query_map(p.as_slice(), |row| Clip::try_from(row))?
            .map(|row| row.map_err(AppError::from))
            .try_collect()
            .context("clip::list paginated")?
    };

    Ok((total_count as usize, clips))
}

pub(crate) fn select_by_group_id(
    db: &DbConn,
    group_id: &models::group::GroupId,
    ShowParams { pagination, filter }: &ShowParams,
) -> Result<(usize, Vec<Clip>)> {
    let sql_filter = sql_filter();

    let mut params = Params::new(filter);

    let total_count: i64 = {
        let mut p = params.as_named();
        p.push((":group_id", group_id));
        db.prepare_cached(&format!(
            "SELECT COUNT(*) FROM clips
            INNER JOIN group_clips ON group_clips.clip_id = clips.id
            WHERE {sql_filter} AND group_clips.group_id = :group_id"
        ))?
        .query_row(p.as_slice(), |row| row.get(0))
        .context("clip::select_by_group_id total_count")?
    };

    let page_size = pagination.page_size_or_default(db)?;
    params.add_pagination(page_size, pagination.page_offset);
    let sort_by: &str = From::from(pagination.sort_by);
    let sort_direction = if pagination.sort_direction_asc {
        "ASC"
    } else {
        "DESC"
    };

    let mut stmt = db.prepare_cached(&format!(
        "SELECT
            c.broadcaster_id AS broadcaster_id,
            c.broadcaster_name AS broadcaster_name,
            c.created_at AS created_at,
            c.creator_name AS creator_name,
            c.duration AS duration,
            c.game_id AS game_id,
            c.id AS id,
            c.lang AS lang,
            c.recorded_at AS recorded_at,
            c.thumbnail_url AS thumbnail_url,
            c.title AS title,
            c.updated_at AS updated_at,
            c.url AS url,
            c.view_count AS view_count,
            c.is_downloaded AS is_downloaded
        FROM clips c
        INNER JOIN group_clips ON group_clips.clip_id = c.id
        WHERE {sql_filter} AND group_clips.group_id = :group_id
        ORDER BY {sort_by} {sort_direction}
        LIMIT :page_size
        OFFSET :page_offset;",
    ))?;
    let clips = {
        let mut p = params.as_named();
        p.push((":group_id", group_id));
        stmt.query_map(p.as_slice(), |row| Clip::try_from(row))?
            .map(|res| res.map_err(AppError::from))
            .try_collect()
            .context("clip::select_by_group_id")?
    };

    Ok((total_count as usize, clips))
}

impl TryFrom<&rusqlite::Row<'_>> for Clip {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> StdResult<Self, Self::Error> {
        Ok(Self {
            broadcaster_id: row.get("broadcaster_id")?,
            broadcaster_name: row.get("broadcaster_name")?,
            created_at: row.get("created_at")?,
            creator_name: row.get("creator_name")?,
            duration: Duration::from_secs(row.get::<_, i64>("duration")? as u64),
            thumbnail_url: row.get("thumbnail_url")?,
            game_id: row.get("game_id")?,
            id: row.get("id")?,
            lang: row.get("lang")?,
            recorded_at: row.get("recorded_at")?,
            title: row.get("title")?,
            updated_at: row.get("updated_at")?,
            url: row.get("url")?,
            view_count: row.get("view_count")?,
            is_downloaded: row.get("is_downloaded")?,
        })
    }
}

fn sql_filter() -> &'static str {
    "
        (:game_id IS NULL OR game_id = :game_id)
        AND (:broadcaster_name IS NULL OR broadcaster_name = :broadcaster_name)
        AND (:title_like IS NULL OR title LIKE '%' || :title_like || '%')
        AND (:skip_langs OR lang IN rarray(:langs))
        AND (:view_count_max IS NULL OR view_count <= :view_count_max)
        AND view_count >= :view_count_min
        AND (:min_recorded_at IS NULL OR recorded_at >= :min_recorded_at)
        AND (:max_recorded_at IS NULL OR recorded_at <= :max_recorded_at)
        AND (:is_downloaded IS NULL OR is_downloaded <= :is_downloaded)
    "
}

struct Params<'a> {
    filter: &'a Filter,
    langs: Rc<Vec<rusqlite::types::Value>>,
    skip_langs: bool,
    page_size_and_offset: Option<(usize, usize)>,
}

impl<'a> Params<'a> {
    fn new(filter: &'a Filter) -> Self {
        let langs = Rc::new(
            filter
                .langs
                .iter()
                .cloned()
                .map(rusqlite::types::Value::from)
                .collect_vec(),
        );

        Self {
            filter,
            langs,
            skip_langs: filter.langs.is_empty(),
            page_size_and_offset: None,
        }
    }

    fn add_pagination(&mut self, page_size: usize, page_offset: usize) {
        self.page_size_and_offset = Some((page_size, page_offset));
    }

    fn as_named(&'a self) -> Vec<(&'static str, &'a dyn ToSql)> {
        let mut p = named_params! {
            ":broadcaster_name": self.filter.broadcaster_name,
            ":title_like": self.filter.title_like,
            ":langs": self.langs,
            ":skip_langs": self.skip_langs,
            ":view_count_max": self.filter.view_count_max,
            ":view_count_min": self.filter.view_count_min,
            ":min_recorded_at": self.filter.min_recorded_at,
            ":max_recorded_at": self.filter.max_recorded_at,
            ":is_downloaded": self.filter.is_downloaded,
        }
        .to_vec();

        if let Some((size, offset)) = &self.page_size_and_offset {
            p.push((":page_size", size));
            p.push((":page_offset", offset));
        }

        p
    }
}

#[cfg(test)]
mod tests {
    //! We have an SQL with some sample data in tests/assets/clips.sql.
    //! Load that data, insert it into in-memory sqlite.

    use crate::models::clip::{Pagination, ShowSortBy};

    use super::*;

    #[test]
    fn it_lists_clips_sorted_asc() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(5),
                    sort_by: ShowSortBy::RecordedAt,
                    sort_direction_asc: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 5);
        assert_eq!(clips[0].id, "FlirtyTenuousCasetteHoneyBadger");
        assert_eq!(clips[1].id, "MoistUnsightlyBatteryTwitchRPG");

        Ok(())
    }

    #[test]
    fn it_lists_clips_sorted_desc() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(1),
                    sort_by: ShowSortBy::RecordedAt,
                    sort_direction_asc: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "DeterminedShyGullKappaWealth");

        Ok(())
    }

    #[test]
    fn it_lists_clips_by_views() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(2),
                    sort_by: ShowSortBy::ViewCount,
                    sort_direction_asc: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "DeterminedShyGullKappaWealth");
        assert_eq!(clips[1].id, "SuaveHonestWeaselJKanStyle");

        Ok(())
    }

    #[test]
    fn it_lists_clips_in_views_range() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(2),
                    sort_by: ShowSortBy::ViewCount,
                    sort_direction_asc: false,
                    ..Default::default()
                },
                filter: Filter {
                    view_count_max: Some(1500),
                    view_count_min: 500,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 4);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "ToughZealousFungusDancingBanana");
        assert_eq!(clips[1].id, "EsteemedShinyAsteriskBibleThump");

        Ok(())
    }

    #[test]
    fn it_lists_clips_with_title_like() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(1),
                    ..Default::default()
                },
                filter: Filter {
                    title_like: Some("pytlik".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 1);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "ToughZealousFungusDancingBanana");

        Ok(())
    }

    #[test]
    fn it_lists_clips_with_offset() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(1),
                    page_offset: 1,
                    sort_by: ShowSortBy::RecordedAt,
                    sort_direction_asc: true,
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 9);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].id, "MoistUnsightlyBatteryTwitchRPG");

        Ok(())
    }

    #[test]
    fn it_lists_clips_based_on_langs() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(10),
                    ..Default::default()
                },
                filter: Filter {
                    langs: vec!["en".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 7);
        assert_eq!(total_count, 7);

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(10),
                    ..Default::default()
                },
                filter: Filter {
                    langs: vec!["nl".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 1);
        assert_eq!(total_count, 1);

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(10),
                    ..Default::default()
                },
                filter: Filter {
                    langs: vec!["nl".to_string(), "cs".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(clips.len(), 2);
        assert_eq!(total_count, 2);

        Ok(())
    }

    #[test]
    fn it_lists_clips_based_on_broadcaster() -> Result<()> {
        let db = prepare_db()?;

        let (total_count, clips) = list(
            &db,
            &GameId::from("55"),
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(2),
                    sort_direction_asc: true,
                    sort_by: ShowSortBy::RecordedAt,
                    ..Default::default()
                },
                filter: Filter {
                    broadcaster_name: Some("Davaeorn".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        assert_eq!(total_count, 5);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "MoistUnsightlyBatteryTwitchRPG");
        assert_eq!(clips[1].id, "EsteemedShinyAsteriskBibleThump");

        Ok(())
    }

    #[test]
    fn it_groups_clips() -> Result<()> {
        let db = prepare_db()?;

        let gid = db::group::insert(&db, "test", "red")?;

        [
            "FlirtyTenuousCasetteHoneyBadger",
            "EsteemedShinyAsteriskBibleThump",
            "MoistUnsightlyBatteryTwitchRPG",
        ]
        .iter()
        .for_each(|clip_id| {
            let clip_id = ClipId::from(*clip_id);
            let manual = false;
            db::group::map_clip(&db, &gid, &clip_id, manual).unwrap();
        });

        let (total_count, clips) = select_by_group_id(
            &db,
            &gid,
            &ShowParams {
                pagination: Pagination {
                    page_size: Some(2),
                    sort_by: ShowSortBy::ViewCount,
                    sort_direction_asc: true,
                    ..Default::default()
                },
                filter: Default::default(),
            },
        )?;
        assert_eq!(total_count, 3);
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "FlirtyTenuousCasetteHoneyBadger");
        assert_eq!(clips[1].id, "MoistUnsightlyBatteryTwitchRPG");

        Ok(())
    }

    fn prepare_db() -> Result<DbConn> {
        pretty_env_logger::try_init_timed().ok();

        let db = db::open(":memory:")?;

        db.execute_batch(include_str!("../../../../tests/assets/clips.sql"))?;

        Ok(db)
    }
}
