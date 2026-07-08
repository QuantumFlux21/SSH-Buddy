# Changelog

All notable changes to SSH-Buddy will be documented in this file.

## Unreleased

### Added

- Server reachability status lights in the left server list.
- Selected-server reachability panel with ICMP ping details, primary TCP port checks, and in-memory latency samples.
- Manual selected-host port scan for a small common-port allowlist.
- Settings action to test the selected external terminal.

### Changed

- Konsole launch now uses `konsole --noclose -e ...` so SSH/SFTP/tunnel/RDP/install-key errors remain visible instead of closing immediately.
- Launch diagnostics now include the full spawned argv preview in addition to the child command preview.

### Security

- Port scanning is manual, selected-host only, and limited to an allowlist. No subnet discovery, background network scanning, credential storage, or shell interpolation was added.

## 0.5.0 - 2026-07-07

Feature release for installing public SSH keys with system OpenSSH tooling.

### Added

- Public key install action using system `ssh-copy-id` in an external terminal.
- Copy install command action generated from saved server profile and SSH key reference data.
- ProxyJump-aware public key install commands using `-o ProxyJump=<value>`.
- Missing `.pub` diagnostics with a recovery command using `ssh-keygen -y -f <private-key> > <private-key>.pub`.
- Launch diagnostics for public key install attempts, including public key path, target user/host/port, ProxyJump, command preview, required binary checks, and preflight status.

### Security

- Public key install uses the matching `<private-key-path>.pub` file only and never reads, stores, or copies private key contents.
- Public key install requires explicit confirmation and does not store remote passwords, SSH key passphrases, or private key contents.

## 0.4.0 - 2026-07-07

Feature release for RDP display/scaling options.

### Added

- RDP native/default scaling mode.
- RDP percentage scaling with allowlisted FreeRDP values: `/scale:100`, `/scale:140`, and `/scale:180`.
- RDP smart sizing through `/smart-sizing`.
- RDP dynamic resolution through `+dynamic-resolution`.
- RDP launch diagnostics now include scaling mode, scaling percent, smart sizing, dynamic resolution, fullscreen, width, and height.

### Changed

- README troubleshooting now includes high-DPI RDP scaling guidance and FreeRDP version/desktop-environment caveats.

## 0.3.1 - 2026-07-07

Bugfix release for Linux AppImage clipboard behavior, launch diagnostics, and RDP interactive prompts.

### Fixed

- Clipboard copy actions now use the Tauri v2 clipboard manager plugin in the desktop app, with a browser clipboard fallback for Vite preview.
- Copy failures now show the generated command in a manual-copy panel instead of only surfacing a platform permission error.
- RDP launch now starts FreeRDP through the selected external terminal so certificate trust and credential prompts have a usable TTY.

### Changed

- SSH, SFTP, tunnel, and RDP launch actions now show a "Last launch attempt" diagnostics panel with the selected terminal/client, executable, command preview, key-file checks, required binary checks, and backend result.
- Launch success messages now clarify that SSH-Buddy started the external terminal/client process, not that the remote connection itself succeeded.
- Linux troubleshooting documentation now calls out KDE/Wayland Konsole behavior and the Alacritty preferred-terminal workaround.
- RDP settings now include an allowlisted certificate mode: default/prompt, trust on first use (`/cert:tofu`), or ignore (`/cert:ignore`, less secure).
- RDP diagnostics now include the FreeRDP executable, terminal-launch status, certificate mode, username/domain metadata, port, multi-monitor status, monitor IDs, and command preview without passwords.
- README troubleshooting now includes KDE/FreeRDP multi-monitor guidance and `xfreerdp3 /monitor-list`.

## 0.3.0 - 2026-07-06

SFTP and RDP external launcher feature release.

### Added

- External terminal SFTP launch using the system OpenSSH `sftp` client.
- Copyable SFTP command generation from saved server profile data.
- SFTP support for username, host, port, identity file references, and ProxyJump.
- Backend tests for SFTP argv construction, command formatting, custom port handling, identity file handling, ProxyJump handling, and launch preflight errors.
- Per-server RDP settings for FreeRDP external launch.
- RDP command generation and launch through `xfreerdp3` or `xfreerdp`.
- RDP options for username, domain, port, fullscreen, multi-monitor, monitor IDs, dimensions, and color depth.
- Backend and frontend tests for RDP settings validation, command construction, client detection, and password-free launch behavior.

### Security

- SFTP launch uses backend-owned argv/process APIs with no shell interpolation.
- SFTP passwords and private key contents remain out of app storage.
- ProxyJump for SFTP is passed as `-o ProxyJump=...` for OpenSSH compatibility.
- RDP launch uses backend-owned argv/process APIs with no shell interpolation.
- RDP passwords are not stored or passed to FreeRDP.
- RDP monitor IDs are validated as comma-separated monitor numbers instead of arbitrary FreeRDP options.
- Arbitrary FreeRDP option strings are not supported in this milestone.

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
