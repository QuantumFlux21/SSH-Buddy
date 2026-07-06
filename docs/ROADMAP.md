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
- Persisted server profiles, groups, tags, SSH key references, ProxyJump values, tunnel profiles, web links, and settings.
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
- Detected `ProxyJump` values are preserved during selected import.

## ProxyJump / Bastion Support

Status: complete for v0.2.0.

- Store optional OpenSSH-compatible `ProxyJump` values on server profiles.
- Validate ProxyJump host specs before save and launch.
- Include `-J <proxy_jump>` in SSH and tunnel argv construction.
- Preserve detected ProxyJump values from `~/.ssh/config` import.
- Keep agent forwarding and password storage out of scope.

## External Terminal Launch

Status: complete.

- Build OpenSSH argv in Rust.
- Launch supported Linux terminals: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, and xterm.
- Preferred terminal setting with auto-detect.
- Copy-command behavior.
- ProxyJump-aware launch behavior.
- Preflight errors for missing OpenSSH, missing supported terminal, and missing selected key files.

## SFTP External Launch

Status: complete for the next release.

- Build OpenSSH `sftp` argv in Rust from saved server profile data.
- Launch `sftp` in an external terminal using the existing terminal launcher.
- Copy SFTP command behavior.
- Include username, host, port, identity file, and ProxyJump.
- Use `-P <port>` for SFTP ports and `-o ProxyJump=<value>` for bastion compatibility.
- Keep FTP, FTPS, password storage, and embedded file browsing out of scope.

## RDP External Launch

Status: complete for the next release.

- Store optional RDP settings per server profile.
- Detect FreeRDP clients in order: `xfreerdp3`, then `xfreerdp`.
- Launch RDP externally through argv/process APIs.
- Copy RDP command behavior.
- Support username, domain, port, fullscreen, multi-monitor, dimensions, and color depth.
- Never store or pass RDP passwords.
- Keep embedded RDP and arbitrary FreeRDP option strings out of scope.

## SSH Tunnels / Port Forwarding

Status: local forwarding complete for v0.2.0.

- Store saved local tunnel profiles per server.
- Launch tunnels with system OpenSSH in an external terminal using `ssh -N -L`.
- Include server profile options such as port, identity file, and ProxyJump.
- Copy tunnel command behavior.
- Validate tunnel labels, host fields, bind hosts, and ports before launch.
- Cascade-delete tunnel profiles when their server profile is deleted.
- Remote forwarding `-R` and SOCKS forwarding `-D` remain post-MVP.

## Web Admin Links

Status: complete.

- Store web links per server.
- Validate only `http://` and `https://` URLs.
- Reject embedded URL credentials.
- Open links through the OS/browser.

## Security Hardening

Status: complete for v0.2.0; release hardening continues.

- Keep command execution backend-owned and narrowly scoped.
- Keep SSH private keys, passphrases, SSH passwords, sudo passwords, and remote passwords out of app storage.
- Use argv/process APIs instead of shell interpolation for SSH, SFTP, RDP, and tunnel launch.
- Keep sudo/root automation out of scope.
- Validate ProxyJump and tunnel values before save and before launch.
- Continue expanding tests around validation, command generation, import, and persistence.

## Release Packaging

Status: complete for v0.1.0; release hardening continues.

- Use GitHub Actions to build platform-specific release artifacts.
- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.
- Do not promise a single universal file across all operating systems.
- One app, one codebase, multiple release files.

## v0.2.0 Release Readiness

Status: ready pending final version bump, release workflow run, and release smoke test.

- Validate ProxyJump and tunnel end-to-end flows.
- Validate GitHub release metadata and keep v0.2.0 marked as a pre-release if appropriate.
- Smoke-test the Linux AppImage on CachyOS/KDE Wayland.
- Track local rolling-release AppImage packaging failures separately from CI release packaging.
- Bump package/app versions to `0.2.0` only in the final release-prep commit before tagging.

## Post-MVP

- Embedded SFTP browser.
- Embedded RDP.
- Remote tunnel forwarding with `-R`.
- SOCKS tunnel forwarding with `-D`.
- Embedded terminal tabs.
- SCP upload/download helper.
- VNC launch.
- Wake-on-LAN.
- Custom command snippets.
- KeePassXC integration.
- Windows/macOS polish.
