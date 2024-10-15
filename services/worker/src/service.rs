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

        // Twitch has two APIs for downloading clips:
        // 1. With the old ones, it's enough to replace some parts of the URL: https://clips-media-assets2.twitch.tv/fbE0GOqCVFURBRSPcGohcg/AT-cm%7CfbE0GOqCVFURBRSPcGohcg-preview-480x272.jpg
        //    --> https://clips-media-assets2.twitch.tv/fbE0GOqCVFURBRSPcGohcg/AT-cm%7CfbE0GOqCVFURBRSPcGohcg.mp4
        //
        //    thumbnail_url.split("-preview-").collect::<Vec<_>>()[0].
        // to_string() + ".mp4"
        //
        // 2. With the new one, a valid signature is needed from Twitch backend.
        //    A signature can be obtained by sending a POST request to the GQL
        //    API, use the clip ID as the slug:
        // curl 'https://gql.twitch.tv/gql#origin=twilight' \
        //     --compressed -X POST \
        //     -H 'Client-Id: kimne78kx3ncx6brgo4mv6wki5h1ko' \
        //     --data-raw
        // '[{"operationName":"ClipsDownloadButton","variables":{"slug":"
        // AbrasiveImpossiblePotatoShadyLulu-2ocNxpYszhvjkCkm"},"extensions":{"
        // persistedQuery":{"version":1,"sha256Hash":"
        // 9c0a5b51612a41b06bfb93065deb6fd7bb7e011db2beb6e5e5d7588ae7f3ff4b"
        // }}}]'
        //
        // This returns a response with signature along the lines of:
        //  "playbackAccessToken":{"signature":"
        // 09f92d3b03734ef50494c9a5e810fed8db762e98","value":"{\"authorization\"
        // :{\"forbidden\":false,\"reason\":\"\"},\"clip_uri\":\"\",\"clip_slug\
        // ":\"AbrasiveImpossiblePotatoShadyLulu-2ocNxpYszhvjkCkm\",\"device_id\
        // ":null,\"expires\":1729059681,\"user_id\":\"\",\"version\":2}"
        //
        // The clip can then be downloaded with
        // GET
        // https://production.assets.clips.twitchcdn.net/v2/media/AbrasiveImpossiblePotatoShadyLulu-2ocNxpYszhvjkCkm/654454a2-8735-4018-b0a8-1ec00d577afc/video.mp4?sig=1e17f3caed84bcde2dbebb9233fbae5b4722e2e0&token={"authorization":{"forbidden":false,"reason":""},"clip_uri":"","clip_slug":"AbrasiveImpossiblePotatoShadyLulu-2ocNxpYszhvjkCkm","device_id":"","expires":1729060527,"user_id":"","version":2}
        //
        // Note that the above links will not work as I removed my user ID and
        // device ID. If you send this request anonymously, those values
        // are not going to be present.
        //
        // We can distinguish between the two APIs by checking the thumbnail
        // URL, as the new ones are going to be in the format:
        // https://static-cdn.jtvnw.net/twitch-clips-thumbnails-prod/AbrasiveImpossiblePotatoShadyLulu-2ocNxpYszhvjkCkm/654454a2-8735-4018-b0a8-1ec00d577afc/preview-480x272.jpg

        Ok(Response::new(()))
    }
}
