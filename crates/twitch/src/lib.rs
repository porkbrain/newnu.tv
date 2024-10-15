//! Interactions with twitch APIs which either would be used by multiple bins
//! or are better organized to be in a dedicated crate.

pub mod models;

pub use twitch_api2;

use anyhow::Result;
use models::GameId;
use std::time::Duration;
use twitch_api2::helix::clips::GetClipsRequest;
use twitch_api2::helix::search::search_categories;
use twitch_api2::helix::Paginated;
use twitch_api2::twitch_oauth2::ClientId;
use twitch_api2::twitch_oauth2::ClientSecret;
use twitch_api2::twitch_oauth2::{AppAccessToken, Scope};
use twitch_api2::TwitchClient;

#[derive(Clone)]
pub struct Client {
    token: AppAccessToken,
    inner: TwitchClient<'static, reqwest::Client>,
}

impl Client {
    /// Go to <https://dev.twitch.tv> to obtain these values.
    pub async fn new(
        twitch_client_id: impl Into<ClientId>,
        twitch_secret: impl Into<ClientSecret>,
    ) -> Result<Self> {
        let inner: TwitchClient<reqwest::Client> = TwitchClient::default();
        let token = AppAccessToken::get_app_access_token(
            &inner,
            twitch_client_id.into(),
            twitch_secret.into(),
            Scope::all(),
        )
        .await?;

        Ok(Self { token, inner })
    }

    /// <https://dev.twitch.tv/docs/api/reference/#search-categories>
    pub async fn search_for_game(
        &self,
        query: &str,
    ) -> Result<Vec<models::Game>> {
        let req = search_categories::SearchCategoriesRequest::builder()
            .query(query)
            .build();

        let resp = self.inner.helix.req_get(req, &self.token).await?;

        Ok(resp.data.into_iter().map(From::from).collect())
    }

    /// <https://dev.twitch.tv/docs/api/reference/#get-games>
    pub async fn get_game(
        &self,
        game_id: GameId,
    ) -> Result<Option<models::Game>> {
        let req =
            twitch_api2::helix::games::get_games::GetGamesRequest::builder()
                .id(vec![game_id.into()])
                .build();

        let resp = self.inner.helix.req_get(req, &self.token).await?;

        Ok(resp.data.into_iter().next().map(From::from))
    }

    /// Performs given request, returning the clips and optionally another
    /// request which contains a cursor for the next page.
    ///
    /// Check for "409" in the requests error to see if you should retry.
    ///
    /// <https://dev.twitch.tv/docs/api/reference/#get-clips>
    pub async fn get_clips_paginated(
        &self,
        req: GetClipsRequest,
    ) -> Result<(Vec<models::Clip>, Option<GetClipsRequest>)> {
        let mut resp =
            self.inner.helix.req_get(req.clone(), &self.token).await?;

        let next_req = if let Some(cursor) = resp.pagination {
            let mut req = resp.request.take().expect("Request missing");
            req.set_pagination(Some(cursor.clone()));
            Some(req)
        } else {
            None
        };

        let clips = resp
            .data
            .into_iter()
            .map(|c| models::Clip {
                url: c.url,
                id: c.id,
                broadcaster_id: c.broadcaster_id.to_string(),
                broadcaster_name: c.broadcaster_name.to_string(),
                creator_name: c.creator_name.to_string(),
                recorded_at: c.created_at.into_string(),
                duration: Duration::from_secs_f64(c.duration),
                title: c.title,
                view_count: c.view_count as usize,
                lang: c.language,
                game_id: c.game_id.to_string(),
                thumbnail_url: c.thumbnail_url,
                video_id: c.video_id.to_string(),
            })
            .collect::<Vec<_>>();

        Ok((clips, next_req))
    }
}
