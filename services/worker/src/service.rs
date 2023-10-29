use std::sync::Arc;

use crate::{job, prelude::*, rpc, RpcWorker};
use rpc::worker_server::Worker;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Worker for RpcWorker {
    async fn add_game(
        &self,
        request: Request<rpc::AddGameRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::AddGameRequest { game_id, game_name } = request.into_inner();
        debug!("Add game {game_id} ({game_name})");

        let db = self.g.db.lock().await;
        db::game::insert(&db, &(game_id.into()), game_name)?;

        Ok(Response::new(()))
    }

    async fn delete_game(
        &self,
        request: Request<rpc::DeleteGameRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::DeleteGameRequest { game_id } = request.into_inner();
        debug!("Delete game {game_id}");

        let db = self.g.db.lock().await;
        db::game::delete(&db, &(game_id.into()))?;

        Ok(Response::new(()))
    }

    async fn set_is_game_paused(
        &self,
        request: Request<rpc::SetIsGamePausedRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::SetIsGamePausedRequest { game_id, is_paused } =
            request.into_inner();
        debug!("Set paused to {is_paused} for game {game_id}");

        let db = self.g.db.lock().await;
        db::game::set_is_paused(&db, &(game_id.into()), is_paused)?;

        Ok(Response::new(()))
    }

    async fn begin_fetch_new_game_clips(
        &self,
        request: Request<rpc::BeginFetchNewGameClipsRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::BeginFetchNewGameClipsRequest {
            game_id,
            recorded_at_most_hours_ago,
            recorded_at_least_hours_ago,
        } = request.into_inner();
        debug!("Begin fetch new game clips for game {game_id}");

        let db = Arc::clone(&self.g.db);
        let tc = Arc::clone(&self.g.twitch);
        tokio::spawn(job::fetch_new_game_clips::once(
            db,
            tc,
            job::fetch_new_game_clips::Conf {
                recorded_at_most_ago: if recorded_at_most_hours_ago == 0 {
                    None
                } else {
                    Some(chrono::Duration::hours(recorded_at_most_hours_ago))
                },
                recorded_at_least_ago: if recorded_at_least_hours_ago == 0 {
                    None
                } else {
                    Some(chrono::Duration::hours(recorded_at_least_hours_ago))
                },
            },
            game_id.into(),
        ));

        Ok(Response::new(()))
    }

    async fn dev_reset(
        &self,
        _request: Request<()>,
    ) -> StdResult<Response<()>, Status> {
        debug!("Dev reset");

        let mut db = self.g.db.lock().await;
        db::down(&mut db)
            .map_err(|e| Status::internal(format!("Failed to down db: {e}")))?;

        db::up(&mut db)
            .map_err(|e| Status::internal(format!("Failed to up db: {e}")))?;

        Ok(Response::new(()))
    }
}
