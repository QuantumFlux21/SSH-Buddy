CREATE TABLE server_tunnels (
  id TEXT PRIMARY KEY,
  server_profile_id TEXT NOT NULL REFERENCES server_profiles(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  tunnel_type TEXT NOT NULL,
  local_bind_host TEXT,
  local_port INTEGER,
  remote_host TEXT,
  remote_port INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_server_tunnels_server_profile_id ON server_tunnels(server_profile_id);
