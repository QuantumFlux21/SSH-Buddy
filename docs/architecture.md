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
- Owns startup window state, tray creation and fallback, close interception, explicit quit, single-instance restoration, and verified OS autostart operations in `desktop.rs`.

## Data Model

- `ServerProfile`: SSH-focused server metadata.
- `Group`: folder-style organization.
- `Tag`: labels for search and filtering.
- `SshKeyRef`: key path and public metadata only.
- `Tunnel`: a saved local (`-L`) forwarding profile associated with a server.
- `RdpSettings`: password-free, allowlisted FreeRDP launch settings associated with a server.
- `WebLink`: named web admin URL attached to a server.
- `AppSettings`: terminal and safety preferences plus the SQLite-owned `startMinimized` and `closeToTray` preferences.

Groups, tags, SSH key references, tunnels, RDP settings, web links, and app settings are stored in the app-owned SQLite database through versioned migrations. Reachability results, port-scan results, and launch diagnostics are transient and are not stored as monitoring history.

Autostart is OS-owned state, not `AppSettings` and not SQLite data. The Rust autostart adapter queries the OS registration after every enable or disable attempt and returns an unknown state with an actionable error when verification is unavailable.

## Desktop Lifecycle

- The configured main window starts hidden and unfocused. Rust chooses normal, minimized-taskbar, or recovery presentation only after migration and initial settings loading.
- A live tray is required before close-to-tray may intercept the main window's close request. Tray or hide failures fall back to normal exit.
- Enabling close-to-tray creates the tray before persistence and removes a newly created tray if persistence fails. Disabling persists first and removes the tray immediately.
- The mandatory tray context menu owns Open and explicit Quit; Linux tray-click events are not required.
- The single-instance plugin is registered first and restores, shows, and focuses the existing window on a second launch.

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

- Frontend Vitest tests cover form helpers, filtering, clipboard behavior, import summaries, tunnels, RDP settings, web links, and immediate desktop-setting behavior with verified post-state handling.
- Rust unit tests cover validation, persistence, integrity/migration backups, the v0.6.0 fixture, SSH config import, command/argv generation, launch outcomes, AppImage restoration, terminal selection/availability, selected-server status checks, and fake-backed desktop lifecycle/autostart decisions.
- CI builds and tests both layers and runs locked `cargo check` jobs on Ubuntu, Windows, and macOS.
