PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE groups (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  color TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE ssh_key_refs (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  path TEXT NOT NULL,
  fingerprint TEXT,
  comment TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE server_profiles (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  host TEXT NOT NULL,
  port INTEGER NOT NULL CHECK (port >= 1 AND port <= 65535),
  username TEXT NOT NULL DEFAULT '',
  identity_file_id TEXT REFERENCES ssh_key_refs(id) ON DELETE SET NULL,
  group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
  notes TEXT,
  favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  proxy_jump TEXT
);

CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE server_profile_tags (
  server_profile_id TEXT NOT NULL REFERENCES server_profiles(id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (server_profile_id, tag_id)
);

CREATE TABLE app_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  terminal_preference TEXT NOT NULL DEFAULT 'auto',
  safety_warnings_enabled INTEGER NOT NULL DEFAULT 1 CHECK (safety_warnings_enabled IN (0, 1))
);

CREATE TABLE server_web_links (
  id TEXT PRIMARY KEY,
  server_profile_id TEXT NOT NULL REFERENCES server_profiles(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  url TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

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
  monitor_ids TEXT,
  certificate_mode TEXT NOT NULL DEFAULT 'prompt',
  scaling_mode TEXT NOT NULL DEFAULT 'native',
  scaling_percent INTEGER,
  FOREIGN KEY (server_profile_id) REFERENCES server_profiles(id) ON DELETE CASCADE
);

CREATE INDEX idx_server_profiles_group_id ON server_profiles(group_id);
CREATE INDEX idx_server_profiles_identity_file_id ON server_profiles(identity_file_id);
CREATE INDEX idx_server_profile_tags_tag_id ON server_profile_tags(tag_id);
CREATE INDEX idx_server_web_links_server_profile_id ON server_web_links(server_profile_id);
CREATE INDEX idx_server_tunnels_server_profile_id ON server_tunnels(server_profile_id);

INSERT INTO schema_migrations (version, applied_at) VALUES
  ('001_initial', '2026-07-01T00:00:00.000Z'),
  ('002_app_settings', '2026-07-01T00:00:01.000Z'),
  ('003_server_web_links', '2026-07-01T00:00:02.000Z'),
  ('004_server_proxy_jump', '2026-07-01T00:00:03.000Z'),
  ('005_server_tunnels', '2026-07-01T00:00:04.000Z'),
  ('006_server_rdp_settings', '2026-07-01T00:00:05.000Z'),
  ('007_rdp_monitor_ids', '2026-07-01T00:00:06.000Z'),
  ('008_rdp_certificate_mode', '2026-07-01T00:00:07.000Z'),
  ('009_rdp_scaling_options', '2026-07-01T00:00:08.000Z');

INSERT INTO groups (id, name, color, created_at, updated_at)
VALUES ('grp_fixture', 'Fixture Lab', '#3aa675', '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z');

INSERT INTO ssh_key_refs (id, label, path, fingerprint, comment, created_at, updated_at)
VALUES ('key_fixture', 'Fixture key', '~/.ssh/id_fixture', 'SHA256:fixture', 'reference only', '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z');

INSERT INTO server_profiles (
  id, display_name, host, port, username, identity_file_id, group_id, notes, favorite,
  created_at, updated_at, proxy_jump
) VALUES (
  'srv_fixture', 'Fixture NAS', 'nas.fixture.invalid', 2222, 'fixture-user', 'key_fixture',
  'grp_fixture', 'non-secret fixture metadata', 1, '2026-07-01T00:01:00.000Z',
  '2026-07-01T00:01:00.000Z', 'jump.fixture.invalid'
);

INSERT INTO tags (id, name, created_at, updated_at)
VALUES ('tag_fixture', 'fixture', '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z');

INSERT INTO server_profile_tags (server_profile_id, tag_id)
VALUES ('srv_fixture', 'tag_fixture');

INSERT INTO app_settings (id, terminal_preference, safety_warnings_enabled)
VALUES (1, 'konsole', 1);

INSERT INTO server_web_links (id, server_profile_id, label, url, created_at, updated_at)
VALUES ('web_fixture', 'srv_fixture', 'Fixture UI', 'https://nas.fixture.invalid', '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z');

INSERT INTO server_tunnels (
  id, server_profile_id, label, tunnel_type, local_bind_host, local_port, remote_host,
  remote_port, created_at, updated_at
) VALUES (
  'tun_fixture', 'srv_fixture', 'Fixture database', 'local', '127.0.0.1', 15432,
  'db.fixture.invalid', 5432, '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z'
);

INSERT INTO server_rdp_settings (
  server_profile_id, enabled, username, domain, port, fullscreen, multi_monitor, width,
  height, color_depth, created_at, updated_at, monitor_ids, certificate_mode, scaling_mode,
  scaling_percent
) VALUES (
  'srv_fixture', 1, 'rdp-fixture', 'FIXTURE', 3389, 0, 1, 1920, 1080, 32,
  '2026-07-01T00:01:00.000Z', '2026-07-01T00:01:00.000Z', '0,1', 'tofu',
  'percentage', 140
);
