ALTER TABLE server_rdp_settings ADD COLUMN scaling_mode TEXT NOT NULL DEFAULT 'native';
ALTER TABLE server_rdp_settings ADD COLUMN scaling_percent INTEGER;
