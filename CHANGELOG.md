# Changelog

All notable changes to SSH-Buddy will be documented in this file.

## 0.1.0 - 2026-07-06

Initial GitHub pre-release.

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
