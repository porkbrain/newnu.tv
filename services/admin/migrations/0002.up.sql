-- stores misc settings as a map
CREATE TABLE settings (
  name TEXT UNIQUE NOT NULL,
  value TEXT NOT NULL
);
-- defaults
INSERT INTO settings (name, value)
VALUES ('recorded_at_least_hours_ago', '8');
INSERT INTO settings (name, value)
VALUES ('fetch_new_game_clips_cron', '@hourly');