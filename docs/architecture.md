# Architecture

SSH-Buddy is split into a React frontend and a Rust backend through Tauri commands.

## Frontend

- Owns layout, forms, search, filtering, and local UI state.
- Calls typed API helpers in `src/lib/api.ts`.
- Performs form validation for immediate user feedback; the backend repeats authoritative validation before persistence or launch.
- Formats copyable command previews from backend-generated command strings and uses the Tauri clipboard plugin with a browser fallback.
- Does not write SQL or construct shell command strings for execution.

## Backend

- Owns SQLite persistence, migrations, authoritative validation, SSH config import, command construction, external process launch, web URL opening, and selected-server reachability checks.
- Stores app data in the OS app data directory.
- Checks SQLite integrity before and after migrations and backs up an existing non-empty database before pending migrations.
- Builds argv arrays for SSH, SFTP, `ssh-copy-id`, FreeRDP, tunnel, terminal-test, and ping processes instead of interpolating shell commands.
- Restores host environment variables for external processes launched from an AppImage.
- Returns launch diagnostics that distinguish missing tools, validation failures, spawn failures, immediate exits, and started terminal processes; it does not claim that a remote connection succeeded.

## Data Model

- `ServerProfile`: SSH-focused server metadata.
- `Group`: folder-style organization.
- `Tag`: labels for search and filtering.
- `SshKeyRef`: key path and public metadata only.
- `Tunnel`: a saved local (`-L`) forwarding profile associated with a server.
- `RdpSettings`: password-free, allowlisted FreeRDP launch settings associated with a server.
- `WebLink`: named web admin URL attached to a server.
- `AppSettings`: terminal and safety preferences.

Groups, tags, SSH key references, tunnels, RDP settings, web links, and app settings are stored in the app-owned SQLite database through versioned migrations. Reachability results, port-scan results, and launch diagnostics are transient and are not stored as monitoring history.

## Connection and Utility Boundaries

- SSH, SFTP, public key install, and local tunnels delegate to system OpenSSH tools.
- RDP delegates to `xfreerdp3` or `xfreerdp` in an external terminal so interactive prompts remain available.
- Public key install requires explicit confirmation and uses the matching `.pub` file; SSH-Buddy does not read or store private key contents.
- Status checks are manual and selected-server only. Port scans use a fixed small allowlist and do not discover subnets.
- Web links are limited to validated `http://` and `https://` URLs without embedded credentials.
- Production and development CSPs keep content and connectivity local except for required Tauri IPC and local development connectivity; clipboard permission is write-only.
- Private keys, passphrases, SSH/SFTP/RDP passwords, sudo passwords, and arbitrary command strings remain outside app storage and launch arguments.

Future connection actions must preserve these backend-owned execution and credential-storage boundaries.

## Tests

- Frontend Vitest tests cover form helpers, filtering, clipboard behavior, import summaries, tunnels, RDP settings, and web links.
- Rust unit tests cover validation, persistence, integrity/migration backups, the v0.6.0 fixture, SSH config import, command/argv generation, launch outcomes, AppImage restoration, terminal selection/availability, and selected-server status checks.
- CI builds and tests both layers and runs `cargo check` for the Tauri backend.
