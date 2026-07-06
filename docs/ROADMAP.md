# Roadmap

## Foundation

Status: complete.

- Tauri 2 desktop scaffold.
- React, TypeScript, and Vite frontend.
- Rust entrypoint and Tauri command registration.
- Linux development script for WebKitGTK/Wayland.
- Baseline README, security policy, and architecture docs.

## SQLite Persistence

Status: complete.

- App-owned SQLite database in the Tauri app data directory.
- Migrations tracked through `schema_migrations`.
- Persisted server profiles, groups, tags, SSH key references, web links, and settings.
- Private keys and passwords stay out of the database.

## Server CRUD

Status: complete.

- Server create, edit, delete, favorite, notes, tags, and search/filtering.
- Group and SSH key reference management.
- Delete confirmation and plaintext-notes warning.

## SSH Config Import

Status: complete.

- Read-only `~/.ssh/config` discovery.
- Preview-first import for concrete `Host` aliases.
- Wildcard/advanced pattern warnings.
- Duplicate detection.
- Selected import only.
- Explicit `IdentityFile` values become key path references, not key contents.

## External Terminal Launch

Status: complete.

- Build OpenSSH argv in Rust.
- Launch supported Linux terminals: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, and xterm.
- Preferred terminal setting with auto-detect.
- Copy-command behavior.
- Preflight errors for missing OpenSSH, missing supported terminal, and missing selected key files.

## Web Admin Links

Status: complete.

- Store web links per server.
- Validate only `http://` and `https://` URLs.
- Reject embedded URL credentials.
- Open links through the OS/browser.

## Security Hardening

Status: in progress.

- Keep command execution backend-owned and narrowly scoped.
- Keep SSH private keys, passphrases, SSH passwords, sudo passwords, and remote passwords out of app storage.
- Use argv/process APIs instead of shell interpolation for SSH launch.
- Keep sudo/root automation out of scope.
- Continue expanding tests around validation, command generation, import, and persistence.

## Release Packaging

Status: planned.

- Use GitHub Actions to build platform-specific release artifacts.
- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.
- Do not promise a single universal file across all operating systems.
- One app, one codebase, multiple release files.

## Post-MVP

- SFTP browser or external SFTP launch.
- RDP launch through `xfreerdp`.
- SSH tunnels and port forwarding.
- Embedded terminal tabs.
- SCP upload/download helper.
- VNC launch.
- Wake-on-LAN.
- Custom command snippets.
- KeePassXC integration.
- Windows/macOS polish.
