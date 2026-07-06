# Architecture

SSH-Buddy is split into a React frontend and a Rust backend through Tauri commands.

## Frontend

- Owns layout, forms, search, filtering, and local UI state.
- Calls typed API helpers in `src/lib/api.ts`.
- Does not write SQL or construct shell command strings for execution.

## Backend

- Owns SQLite persistence, migrations, validation, SSH config import, SSH command construction, external terminal launch, and web URL opening.
- Stores app data in the OS app data directory.
- Uses argv arrays for process execution.

## Data Model

- `ServerProfile`: SSH-focused server metadata.
- `Group`: folder-style organization.
- `Tag`: labels for search and filtering.
- `SshKeyRef`: key path and public metadata only.
- `WebLink`: named web admin URL attached to a server.
- `AppSettings`: terminal and safety preferences.

Only SSH launch, copy-command, and web/admin links are implemented for the MVP. Future connection actions should be added without storing remote passwords or bypassing OpenSSH/OS security controls.
