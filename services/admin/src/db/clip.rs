use anyhow::Context;
use itertools::Itertools;
use rusqlite::named_params;
use std::{rc::Rc, time::Duration};
use twitch::models::GameId;

use crate::models::clip::{Clip, Filter, Pagination, ShowParams};
use crate::prelude::*;

pub fn list(
    db: &DbConn,
    game_id: &GameId,
    ShowParams { pagination, filter }: &ShowParams,
) -> Result<(usize, Vec<Clip>)> {
    let Filter {
        broadcaster_name,
        langs,
        max_recorded_at,
        min_recorded_at,
        title_like,
        view_count_max,
        view_count_min,
    } = filter;
    let Pagination {
        page_offset,
        page_size,
        sort_by,
        sort_direction_asc,
    } = pagination;

    let page_size = match page_size {
        Some(0) => {
            return Err(AppError::bad_request(
                "page_size must be greater than 0".to_string(),
            ));
        }
        Some(page_size) => *page_size,
        None => db::setting::default_page_size(&db)?,
    };

    let sort_by: &str = From::from(*sort_by);
    let sort_direction = if *sort_direction_asc { "ASC" } else { "DESC" };
    // array feature of sqlite
    let langs = Rc::new(
        langs
            .iter()
            .cloned()
            .map(rusqlite::types::Value::from)
            .collect_vec(),
    );
    let where_clause = "WHERE
        game_id = :game_id
        AND (:broadcaster_name IS NULL OR broadcaster_name = :broadcaster_name)
        AND (:title_like IS NULL OR title LIKE '%' || :title_like || '%')
        AND (:skip_langs OR lang IN rarray(:langs))
        AND (:view_count_max IS NULL OR view_count <= :view_count_max)
        AND view_count >= :view_count_min
        AND (:min_recorded_at IS NULL OR recorded_at >= :min_recorded_at)
        AND (:max_recorded_at IS NULL OR recorded_at <= :max_recorded_at)";

    let total_count_sql = format!("SELECT COUNT(*) FROM clips {where_clause}");
    let params = named_params! {
        ":broadcaster_name": broadcaster_name,
        ":game_id": game_id,
        ":langs": langs,
        ":max_recorded_at": max_recorded_at,
        ":min_recorded_at": min_recorded_at,
        ":skip_langs": langs.is_empty(),
        ":title_like": title_like,
        ":view_count_max": view_count_max,
        ":view_count_min": view_count_min,
    };
    let total_count: i64 = db
        .prepare(&total_count_sql)?
        .query_row(params, |row| row.get(0))
        .context("Total clips")?;

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
            view_count
        FROM clips
        {where_clause}
        ORDER BY {sort_by} {sort_direction}
        LIMIT :page_size
        OFFSET :page_offset"
    );
    let params = named_params! {
        ":broadcaster_name": broadcaster_name,
        ":game_id": game_id,
        ":langs": langs,
        ":max_recorded_at": max_recorded_at,
        ":min_recorded_at": min_recorded_at,
        ":page_offset": page_offset,
        ":page_size": page_size,
        ":skip_langs": langs.is_empty(),
        ":title_like": title_like,
        ":view_count_max": view_count_max,
        ":view_count_min": view_count_min,
    };

    let clips = db
        .prepare_cached(&paginated_sql)?
        .query_map(params, |row| Clip::try_from(row))?
        .map(|row| row.map_err(AppError::from))
        .try_collect()
        .context("List paginated clips")?;

    Ok((total_count as usize, clips))
}

pub fn select_by_group_id(
    db: &DbConn,
    group_id: &models::group::GroupId,
    query: &models::clip::Pagination,
) -> Result<(usize, Vec<Clip>)> {
    let page_size = match query.page_size {
        Some(0) => {
            return Err(AppError::bad_request(
                "page_size must be greater than 0".to_string(),
            ));
        }
        Some(page_size) => page_size,
        None => db::setting::default_page_size(&db)?,
    };

    let total_count: i64 = db
        .prepare_cached(&format!(
            "SELECT COUNT(*) FROM clips
            INNER JOIN group_clips ON group_clips.clip_id = clips.id
            WHERE group_clips.group_id = :group_id"
        ))?
        .query_row(
            named_params! {
                ":group_id": group_id,
            },
            |row| row.get(0),
        )
        .context("clip::select_by_group_id total_count")?;

    let sort_by: &str = From::from(query.sort_by);
    let sort_direction = if query.sort_direction_asc {
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
            c.view_count AS view_count
        FROM clips c
        INNER JOIN group_clips ON group_clips.clip_id = c.id
        WHERE group_clips.group_id = :group_id
        ORDER BY {sort_by} {sort_direction}
        LIMIT :page_size
        OFFSET :page_offset;",
    ))?;
    let clips = stmt
        .query_map(
            named_params! {
                ":group_id": group_id,
                ":page_size": page_size,
                ":page_offset": query.page_offset,
            },
            |row| Clip::try_from(row),
        )?
        .map(|res| res.map_err(AppError::from))
        .try_collect()
        .context("clip::select_by_group_id")?;

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
        })
    }
}

#[cfg(test)]
mod tests {
    //! We have an SQL with some sample data in tests/assets/clips.sql.
    //! Load that data, insert it into in-memory sqlite.

    use crate::models::clip::ShowSortBy;

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
            &Pagination {
                page_size: Some(2),
                sort_by: ShowSortBy::ViewCount,
                sort_direction_asc: true,
                ..Default::default()
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
