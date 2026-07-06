# Roadmap

## Foundation

Status: complete.

- Tauri 2 desktop scaffold.
- React, TypeScript, and Vite frontend.
- Rust entrypoint and Tauri command registration.
- Linux development script for WebKitGTK/Wayland.
- Baseline README, security policy, and CI workflow.

## SQLite Persistence

Status: next.

- Add app-owned SQLite database and migrations.
- Persist server profiles, groups, tags, key references, web links, connection actions, and settings.
- Keep private keys and passwords out of the database.

## Server CRUD

Status: planned.

- Server, group, tag, key reference CRUD.
- Notes and search/filtering.

## SSH Config Import

Status: planned.

- `~/.ssh/config` import preview.
- Concrete host import only for the first version.
- Duplicate handling and import warnings.

## External Terminal Launch

Status: planned.

- Build OpenSSH argv in Rust.
- Launch preferred Linux terminal.
- Keep copy-command behavior available.

## Web Admin Links

Status: planned.

- Store web links per server.
- Validate `http://` and `https://` URLs.
- Open links through the OS/browser.

## Security Hardening

Status: planned.

- Keep command execution backend-owned and narrowly scoped.
- Add safety warnings for root login, agent forwarding, password storage, and privileged command automation.
- Expand tests around validation, command generation, and persistence.

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
- VNC launch, Wake-on-LAN, custom command snippets, KeePassXC integration, and Windows/macOS polish.
