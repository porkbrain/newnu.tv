use anyhow::Context;
use itertools::Itertools;
use rusqlite::{named_params, ErrorCode};

use crate::{models::group::Group, prelude::*};

pub fn insert(db: &DbConn, name: &str, color: &str) -> Result<GroupId> {
    let res = db.execute(
        "INSERT INTO
            groups (name, color)
        VALUES
            (:name, :color);",
        named_params! {
            ":name": name,
            ":color": color,
        },
    );
    let Err(e) = res else {
        return Ok(select_by_name(db, name)?.id);
    };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Group '{name}' already exists"),
        )),
        _ => Err(AppError::internal(e.to_string())),
    }
}

pub fn update_metadata_json(
    db: &DbConn,
    id: &GroupId,
    metadata_json: serde_json::Value,
) -> Result<()> {
    db.execute(
        "UPDATE groups
        SET metadata_json = :metadata_json
        WHERE id = :id;",
        named_params! {
            ":id": id,
            ":metadata_json": serde_json::to_string(&metadata_json)?,
        },
    )?;

    Ok(())
}

pub fn select_all(db: &DbConn) -> Result<Vec<Group>> {
    let mut stmt = db.prepare_cached(
        "SELECT
            id,
            name,
            color,
            metadata_json,
            zindex,
            archived,
            download_enabled,
            created_at
        FROM groups
        WHERE archived = false
        ORDER BY zindex DESC, name ASC;",
    )?;
    let groups = stmt
        .query_map((), |row| Group::try_from(row))?
        .map(|res| res.map_err(AppError::from))
        .collect::<Result<_>>()?;
    Ok(groups)
}

pub fn select_all_with_metadata(db: &DbConn) -> Result<Vec<Group>> {
    let mut stmt = db.prepare_cached(
        "SELECT
            id,
            name,
            color,
            metadata_json,
            zindex,
            archived,
            download_enabled,
            created_at
        FROM groups
        WHERE archived = false
        AND metadata_json IS NOT NULL;",
    )?;
    let groups = stmt
        .query_map([], |row| Group::try_from(row))?
        .map(|row| row.map_err(AppError::from))
        .try_collect()?;

    Ok(groups)
}

pub fn select_by_name(db: &DbConn, name: &str) -> Result<Group> {
    let mut stmt = db.prepare_cached(
        "SELECT
            id,
            name,
            color,
            metadata_json,
            zindex,
            archived,
            download_enabled,
            created_at
        FROM groups WHERE name = :name;",
    )?;
    let group = stmt.query_row(named_params! { ":name": name }, |row| {
        Group::try_from(row)
    })?;
    Ok(group)
}

pub fn select_by_id(db: &DbConn, id: &GroupId) -> Result<Group> {
    let mut stmt = db.prepare_cached(
        "SELECT
            id,
            name,
            color,
            metadata_json,
            zindex,
            archived,
            download_enabled,
            created_at
        FROM groups WHERE id = :id;",
    )?;
    let group = stmt
        .query_row(named_params! { ":id": id }, |row| Group::try_from(row))
        .context("group::select_by_id")?;
    Ok(group)
}

pub fn map_clip(
    db: &DbConn,
    group_id: &GroupId,
    clip_id: &ClipId,
    manual: bool,
) -> Result<()> {
    db.execute(
        "INSERT INTO
            group_clips (clip_id, group_id, manually_created)
        VALUES
            (:clip_id, :group_id, :manually_created)
        ON CONFLICT (clip_id, group_id) DO NOTHING
        ;",
        named_params! {
            ":clip_id": clip_id,
            ":group_id": group_id,
            ":manually_created": manual,
        },
    )?;

    Ok(())
}

pub fn archive(db: &DbConn, id: &GroupId) -> Result<()> {
    db.execute(
        "UPDATE groups
        SET archived = true
        WHERE id = :id;",
        named_params! {
            ":id": id,
        },
    )?;

    Ok(())
}

pub fn set_auto_download(
    db: &DbConn,
    id: &GroupId,
    enable: bool,
) -> Result<()> {
    db.execute(
        "UPDATE groups
        SET download_enabled = :enable
        WHERE id = :id;",
        named_params! {
            ":id": id,
            ":enable": enable,
        },
    )?;

    Ok(())
}

impl TryFrom<&rusqlite::Row<'_>> for Group {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> StdResult<Self, Self::Error> {
        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            color: row.get("color")?,
            metadata_json: row.get("metadata_json")?,
            zindex: row.get("zindex")?,
            archived: row.get("archived")?,
            download_enabled: row.get("download_enabled")?,
            created_at: row.get("created_at")?,
        })
    }
}
