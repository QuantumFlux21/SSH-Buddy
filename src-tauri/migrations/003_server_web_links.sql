CREATE TABLE server_web_links (
  id TEXT PRIMARY KEY,
  server_profile_id TEXT NOT NULL REFERENCES server_profiles(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  url TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_server_web_links_server_profile_id ON server_web_links(server_profile_id);
