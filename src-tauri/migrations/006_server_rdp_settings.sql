CREATE TABLE server_rdp_settings (
  server_profile_id TEXT PRIMARY KEY NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 0,
  username TEXT,
  domain TEXT,
  port INTEGER NOT NULL DEFAULT 3389,
  fullscreen INTEGER NOT NULL DEFAULT 0,
  multi_monitor INTEGER NOT NULL DEFAULT 0,
  width INTEGER,
  height INTEGER,
  color_depth INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (server_profile_id) REFERENCES server_profiles(id) ON DELETE CASCADE
);
