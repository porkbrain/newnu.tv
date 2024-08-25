-- add a 'download_enabled' flag which determines whether the worker should
-- be instructed to automatically download clips for this group
ALTER TABLE groups
ADD COLUMN download_enabled BOOLEAN NOT NULL DEFAULT FALSE;
-- every 30 min, download clips for groups which have download_enabled set to
INSERT INTO settings (name, value)
VALUES (
        'download_clips_cron',
        -- sec   min     hour   day of month   month   day of week   year
        '0      0,30     *      *              *       *             *'
    );
-- add a 'is_downloaded' flag to clips table which informs whether the clips
-- has been persisted by the worker
ALTER TABLE clips
ADD COLUMN is_downloaded BOOLEAN NOT NULL DEFAULT FALSE;