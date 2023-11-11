CREATE TABLE IF NOT EXISTS clips (
    -- twitch id of the clip
    id TEXT NOT NULL UNIQUE,
    -- as returned from twitch APIs
    broadcaster_id TEXT NOT NULL,
    -- this is the display name and will differ from the login name by
    -- capitalization
    broadcaster_name TEXT NOT NULL,
    -- the user who created the clip
    creator_name TEXT NOT NULL,
    -- RFC3993 format of when the video was recorded, also called `created_at`
    -- in the Twitch APIs
    recorded_at TEXT NOT NULL,
    -- how many seconds long is the clip
    duration INTEGER NOT NULL,
    -- the user generated title of the clip
    -- unsafe!
    title TEXT,
    -- the url to the thumbnail image
    thumbnail_url TEXT NOT NULL,
    -- where can we download the video
    url TEXT NOT NULL,
    -- the view count at the time of updated_at
    view_count INTEGER NOT NULL,
    -- twitch language code
    -- can be empty string for unsupported langs
    lang TEXT NOT NULL DEFAULT 'en',
    -- not a foreign key, but can be joined with games table using this
    game_id TEXT NOT NULL,
    -- last time this clip was updated in the database
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    -- when was the clip created in the database
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);