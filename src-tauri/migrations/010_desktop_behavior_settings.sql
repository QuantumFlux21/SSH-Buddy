ALTER TABLE app_settings ADD COLUMN start_minimized INTEGER NOT NULL DEFAULT 0 CHECK (start_minimized IN (0, 1));
ALTER TABLE app_settings ADD COLUMN close_to_tray INTEGER NOT NULL DEFAULT 0 CHECK (close_to_tray IN (0, 1));
