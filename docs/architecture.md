# Architecture

SSH-Buddy is split into a React frontend and a Rust backend through Tauri commands.

## Frontend

- Owns layout, forms, search, filtering, and local UI state.
- Calls typed API helpers in `src/lib/api.ts`.
- Does not write SQL or construct shell command strings for execution.

## Backend

- Will own SQLite persistence, migrations, validation, SSH config import, SSH command construction, external terminal launch, and web URL opening.
- Will store app data in the OS app data directory.
- Uses argv arrays for process execution.

## Data Model

- `ServerProfile`: SSH-focused server metadata.
- `Group`: folder-style organization.
- `Tag`: labels for search and filtering.
- `SshKeyRef`: key path and public metadata only.
- `ConnectionAction`: extensible action record.
- `WebLink`: named web admin URL attached to a server.
- `AppSettings`: terminal and safety preferences.

Only `ssh`, `copy-command`, and `web` actions are planned for the MVP.
