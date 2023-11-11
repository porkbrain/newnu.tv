CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY,
    -- human friendly name
    name TEXT NOT NULL UNIQUE,
    -- used in UI
    color TEXT NOT NULL,
    -- additional metadata is unstructured on the db level to speed up
    -- development
    metadata_json TEXT,
    -- will be used for ordering groups in the UI, higher is better
    zindex INTEGER NOT NULL DEFAULT 0,
    -- when was the group created
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
-- maps clips to groups
CREATE TABLE IF NOT EXISTS group_clips (
    group_id INTEGER NOT NULL,
    clip_id TEXT NOT NULL,
    manually_created BOOLEAN NOT NULL,
    -- when was the clip added to the group
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (group_id, clip_id)
);
-- every 10 minutes, map clips to groups
INSERT INTO settings (name, value)
VALUES (
        'map_clips_to_groups_cron',
        -- sec   min                 hour   day of month   month   day of week   year
        '0      0,10,20,30,40,50     *      *              *       *             *'
    );
INSERT INTO settings (name, value)
VALUES ('default_page_size', '100');