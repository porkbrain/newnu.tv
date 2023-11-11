use crate::{prelude::*, rpc, RpcWorker};
use rpc::worker_server::Worker;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Worker for RpcWorker {
    async fn download_clips(
        &self,
        request: Request<rpc::DownloadClipsRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::DownloadClipsRequest { clips } = request.into_inner();
        debug!("Download {} clips", clips.len());

        // TODO

        Ok(Response::new(()))
    }
}
