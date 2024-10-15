use crate::prelude::*;
use chrono::Utc;
use rusqlite::{named_params, ErrorCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Game {
    pub(crate) id: twitch::models::GameId,
    pub(crate) name: String,
    /// Note that this _might_ contain a placeholder for the size.
    ///
    /// `https://static-cdn.jtvnw.net/ttv-boxart/{game_id}-{width}x{height}.jpg`
    pub(crate) box_art_url: String,
    pub(crate) is_paused: bool,
}

pub(crate) fn insert(db: &DbConn, game: &twitch::models::Game) -> Result<()> {
    let res = db.execute(
        "INSERT INTO
            games (id, name, box_art_url)
        VALUES
            (:id, :name, :box_art_url);",
        named_params! {
            ":id": game.id,
            ":name": game.name,
            ":box_art_url": game.box_art_url,
        },
    );
    let Err(e) = res else { return Ok(()) };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Game with id {} already exists", game.id),
        )),
        _ => Err(AppError::internal(e.to_string())),
    }
}

pub(crate) fn delete(
    db: &DbConn,
    game_id: &twitch::models::GameId,
) -> Result<()> {
    db.execute(
        "DELETE FROM games WHERE id = :id;",
        named_params! { ":id": game_id },
    )
    .map(drop)
    .map_err(From::from)
}

pub(crate) fn select_all(db: &DbConn) -> Result<Vec<Game>> {
    let mut stmt = db.prepare_cached(
        "SELECT id, name, box_art_url, is_paused
        FROM games ORDER BY is_paused ASC, name ASC;",
    )?;
    let games = stmt
        .query_map((), |row| Game::try_from(row))?
        .map(|res| res.map_err(AppError::from))
        .collect::<Result<_>>()?;
    Ok(games)
}

pub(crate) fn select_by_id(
    db: &DbConn,
    game_id: &twitch::models::GameId,
) -> Result<Game> {
    let mut stmt = db.prepare_cached(
        "SELECT id, name, box_art_url, is_paused
        FROM games WHERE id = :id;",
    )?;
    let game = stmt.query_row(named_params! { ":id": game_id }, |row| {
        Game::try_from(row)
    })?;
    Ok(game)
}

pub(crate) fn set_is_paused(
    db: &DbConn,
    game_id: &twitch::models::GameId,
    is_paused: bool,
) -> Result<()> {
    let res = db.execute(
        "UPDATE games SET is_paused = :is_paused WHERE id = :game_id;",
        named_params! { ":game_id": game_id, ":is_paused": is_paused },
    );
    let Err(e) = res else { return Ok(()) };

    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => Err(AppError::already_exists(
            format!("Game with id {game_id} does not exist"),
        )),
        _ => Err(AppError::internal(e.to_string())),
    }
}

#[derive(Debug)]
pub(crate) struct GameWithLatestClipRecordedAt {
    pub(crate) game_id: twitch::models::GameId,
    pub(crate) latest_clip_recorded_at: Option<chrono::DateTime<Utc>>,
}

pub(crate) fn select_all_active_that_have_last_clip_older_than(
    db: &DbConn,
    duration: chrono::Duration,
) -> Result<Vec<GameWithLatestClipRecordedAt>> {
    db.prepare(
        "
            SELECT
                games.id as game_id,
                MAX(clips.recorded_at) as latest_clip_recorded_at
            FROM games
            LEFT JOIN clips ON clips.game_id = games.id
            WHERE games.is_paused = FALSE
            AND games.id NOT IN (
                SELECT game_id
                FROM clips
                WHERE recorded_at > :last_queried
            )
            GROUP BY games.id
        ",
    )?
    .query_map(
        named_params! { ":last_queried": chrono::Utc::now() - duration },
        |row| {
            Ok(GameWithLatestClipRecordedAt {
                game_id: row.get("game_id")?,
                latest_clip_recorded_at: row.get("latest_clip_recorded_at")?,
            })
        },
    )?
    .map(|res| res.map_err(AppError::from))
    .collect()
}

pub(crate) fn select_latest_clip_recorded_at(
    db: &DbConn,
    game_id: &twitch::models::GameId,
) -> Result<Option<chrono::DateTime<Utc>>> {
    db.prepare(
        "
            SELECT
                MAX(clips.recorded_at) as latest_clip_recorded_at
            FROM clips
            WHERE clips.game_id = :game_id
        ",
    )?
    .query_row(named_params! { ":game_id": game_id }, |row| {
        row.get("latest_clip_recorded_at")
    })
    .map_err(AppError::from)
}

impl TryFrom<&rusqlite::Row<'_>> for Game {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> StdResult<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            box_art_url: twitch::models::into_standard_box_art_size(
                row.get::<_, String>("box_art_url")?,
            ),
            is_paused: row.get("is_paused")?,
        })
    }
}
