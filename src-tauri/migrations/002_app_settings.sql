CREATE TABLE app_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  terminal_preference TEXT NOT NULL DEFAULT 'auto',
  safety_warnings_enabled INTEGER NOT NULL DEFAULT 1 CHECK (safety_warnings_enabled IN (0, 1))
);

INSERT INTO app_settings (id, terminal_preference, safety_warnings_enabled)
VALUES (1, 'auto', 1);
