# Changelog

All notable changes to SSH-Buddy will be documented in this file.

## Unreleased

### Added

- External terminal SFTP launch using the system OpenSSH `sftp` client.
- Copyable SFTP command generation from saved server profile data.
- SFTP support for username, host, port, identity file references, and ProxyJump.
- Backend tests for SFTP argv construction, command formatting, custom port handling, identity file handling, ProxyJump handling, and launch preflight errors.

### Security

- SFTP launch uses backend-owned argv/process APIs with no shell interpolation.
- SFTP passwords and private key contents remain out of app storage.
- ProxyJump for SFTP is passed as `-o ProxyJump=...` for OpenSSH compatibility.

## 0.2.0 - 2026-07-06

ProxyJump and local SSH tunnel feature release.

### Added

- ProxyJump/bastion support on server profiles with OpenSSH-compatible validation.
- SSH launch support for ProxyJump through `ssh -J`.
- SSH config import now preserves detected `ProxyJump` values for selected imports.
- Saved local SSH tunnel profiles per server.
- Tunnel command generation and copy behavior using `ssh -N -L`.
- External-terminal tunnel launch through the existing safe terminal launcher.
- Backend and frontend tests for ProxyJump, tunnel validation, tunnel command construction, persistence, and import behavior.
- v0.2.0 release notes.

### Documentation

- Add post-release install instructions for GitHub Releases, Linux AppImage execution, Wayland/WebKit troubleshooting, and local app data reset.
- Clarify where SSH-Buddy stores its local SQLite database.
- Update release-readiness notes for v0.2.0 feature release preparation.
- Document the local rolling-release Linux AppImage packaging caveat.
- Document ProxyJump and local tunnel usage.
- Document ProxyJump and tunnel security boundaries.

### Changed

- Make the release workflow draft text generic for future `v*` tags instead of hardcoding v0.1.0 wording.
- Keep remote forwarding `-R`, SOCKS forwarding `-D`, SFTP, RDP, embedded terminal, sync, password storage, and KeePassXC out of scope.

## 0.1.0 - 2026-07-06

Initial public GitHub release.

### Added

- Tauri 2, React, TypeScript, Rust, and SQLite desktop app foundation.
- Local SQLite persistence for server profiles, groups, tags, SSH key references, web/admin links, and app settings.
- Server CRUD with favorites, notes, tags, search, filtering, delete confirmation, and dark UI.
- SSH key reference management that stores key paths and optional public metadata only.
- External terminal SSH launch through system OpenSSH.
- Copyable SSH command generation.
- Web/admin links with `http://` and `https://` validation.
- Read-only `~/.ssh/config` import preview and selected import.
- Import warnings for wildcard/advanced Host patterns, duplicates, OpenSSH resolution issues, and ProxyJump.
- Security documentation covering key handling, password storage boundaries, and sudo/root limitations.
- GitHub Actions CI and tagged pre-release packaging workflows.

### Known Limitations

- Linux-first release target; Windows and macOS artifacts are experimental until tested on real machines.
- Builds are unsigned; Windows and macOS may show operating system trust warnings.
- SSH config import handles concrete `Host` aliases only and does not edit `~/.ssh/config`.
- ProxyJump is detected in import preview but not stored or used for launch yet.
- No SFTP, RDP, tunnels, embedded terminal, sync, KeePassXC, or sudo/root automation.
