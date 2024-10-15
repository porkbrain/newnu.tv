use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Clip {
    /// Twitch assigned id
    pub id: String,
    /// Twitch assigned id
    pub broadcaster_id: String,
    /// Display name is case insensitively equal to the twitch broadcaster
    /// login
    pub broadcaster_name: String,
    /// The name of the user who recorded the clip
    pub creator_name: String,
    /// When the clip was created on Twitch side
    pub recorded_at: String,
    /// We store only seconds in db
    pub duration: Duration,
    /// Title of the clip as set by the creator, hence unsafe
    pub title: String,
    /// Links to the Twitch clips website where the clip can be viewed.
    /// This is not the direct link to the clip mp4.
    pub url: String,
    /// The image url for thumbnail
    pub thumbnail_url: String,
    /// The number of times the clip has been viewed.
    /// This number depends on when we queried the APIs.
    pub view_count: usize,
    /// Can be empty if Twitch doesn't recognize the language
    pub lang: String,
    /// Twitch assigned id
    pub game_id: String,
    /// ID of the video from which the clip was created.
    pub video_id: Option<String>,
}
