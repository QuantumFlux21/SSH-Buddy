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

Status: complete for v0.3.0.

- Build OpenSSH `sftp` argv in Rust from saved server profile data.
- Launch `sftp` in an external terminal using the existing terminal launcher.
- Copy SFTP command behavior.
- Include username, host, port, identity file, and ProxyJump.
- Use `-P <port>` for SFTP ports and `-o ProxyJump=<value>` for bastion compatibility.
- Keep FTP, FTPS, password storage, and embedded file browsing out of scope.

## Public Key Install

Status: complete for v0.5.0.

- Build `ssh-copy-id` argv from a saved server profile and SSH key reference.
- Resolve and preflight the matching `<private-key-path>.pub` file without reading private key contents.
- Require explicit confirmation before launch.
- Keep password and host-key prompts in the external terminal.
- Support saved SSH ports and validated ProxyJump values.
- Keep remote password, passphrase, private key content, and sudo/root automation out of app storage and arguments.

## RDP External Launch

Status: complete for v0.3.0.

- Store optional RDP settings per server profile.
- Detect FreeRDP clients in order: `xfreerdp3`, then `xfreerdp`.
- Launch RDP externally through argv/process APIs.
- Copy RDP command behavior.
- Support username, domain, port, fullscreen, multi-monitor, monitor IDs, dimensions, and color depth.
- Never store or pass RDP passwords.
- Keep embedded RDP and arbitrary FreeRDP option strings out of scope.

## RDP Display and Scaling

Status: complete for v0.4.0.

- Support allowlisted native, percentage, smart-sizing, and dynamic-resolution modes.
- Support validated multi-monitor IDs, dimensions, and color depth.
- Keep certificate handling allowlisted to default/prompt, trust on first use, or explicit ignore.
- Keep RDP passwords and arbitrary FreeRDP option strings out of scope.

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

## Launch and Clipboard Diagnostics

Status: complete in the unreleased candidate folded into v0.7.0.

- Use the Tauri clipboard plugin in the desktop app with a browser fallback for frontend preview.
- Show a manual-copy fallback when clipboard access fails.
- Report command previews, resolved executables, key-path checks, required-binary checks, final argv, and started/not-found/validation/spawn/immediate-exit outcomes.
- Launch Konsole with `--separate --noclose -e` so launch behavior is isolated and child-process errors remain visible.
- Report host terminal availability and provide a harmless test of the currently visible selection.
- Restore the host launch environment for external tools started from an AppImage.
- Bound and sanitize immediate-exit stderr without claiming that a remote connection succeeded.

## Database Recovery

Status: complete in the unreleased candidate folded into v0.7.0.

- Check SQLite integrity before and after migrations.
- Create a recoverable pre-migration backup before modifying an existing non-empty database with pending migrations.
- Preserve v0.6.0 profiles, groups, tags, key references, web links, tunnels, RDP settings, and application settings.
- Disable database access for the session after a startup failure and avoid suggesting that existing data was deleted.

## Selected-Server Status

Status: complete for v0.6.0.

- Run manual ping and primary TCP-port checks for the selected server.
- Show online, degraded, offline, checking, and unknown states without background polling.
- Keep recent latency samples in memory only.
- Limit manual port scans to the selected host and a fixed common-port allowlist.
- Keep subnet discovery, arbitrary port ranges, background scanning, and monitoring history out of scope.

## Desktop Behavior

Status: implemented for v0.7.0; release acceptance remains outstanding.

- Persist start-minimized and close-to-tray preferences in migration 010 with defaults off.
- Keep login startup OS-owned and report the verified state after every change attempt.
- Delay initial window presentation until database migration and settings loading finish.
- Provide normal, minimized-taskbar, and recovery startup behavior with safe window-operation fallbacks.
- Require a live tray and mandatory Open/Quit context menu before intercepting close.
- Restore hidden or minimized windows after a second process launch.
- Keep automated autostart tests fake-backed so runner login configuration is never modified.

## Security Hardening

Status: complete for v0.3.0; release hardening continues.

- Keep command execution backend-owned and narrowly scoped.
- Keep SSH private keys, passphrases, SSH passwords, sudo passwords, and remote passwords out of app storage.
- Use argv/process APIs instead of shell interpolation for SSH, SFTP, RDP, and tunnel launch.
- Keep sudo/root automation out of scope.
- Validate ProxyJump, tunnel, SFTP, and RDP values before save and before launch.
- Validate SSH host and username fields before save, SSH config resolution/import, command display, and launch.
- Keep production/development CSP connectivity local-only except for required Tauri IPC and local development connectivity.
- Keep clipboard permission write-only and external URL opening backend-owned and validated.
- Keep arbitrary FreeRDP option strings and RDP password handling out of scope.
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

Status: complete.

- Validate ProxyJump and tunnel end-to-end flows.
- Validate GitHub release metadata and keep v0.2.0 marked as a pre-release if appropriate.
- Smoke-test the Linux AppImage on CachyOS/KDE Wayland.
- Track local rolling-release AppImage packaging failures separately from CI release packaging.
- Bump package/app versions to `0.2.0` only in the final release-prep commit before tagging.

## v0.3.0 Release Readiness

Status: complete.

- Validate SFTP external launch and copy-command behavior.
- Validate RDP external launch and copy-command behavior.
- Validate RDP username, domain, port, fullscreen, dimensions, color depth, multi-monitor, and monitor ID settings.
- Confirm RDP monitor IDs are passed as `/monitors:<ids>` only after validation and are not arbitrary FreeRDP options.
- Confirm SFTP/RDP password storage remains out of scope.
- Smoke-test the Linux AppImage on CachyOS/KDE Wayland when available.
- Track local rolling-release AppImage packaging failures separately from CI release packaging.
- Version package/app metadata consistently before tagging.

## v0.4.0 through v0.6.0 Release Readiness

Status: complete.

- Shipped allowlisted RDP display/scaling settings in v0.4.0.
- Shipped explicit public key install through system `ssh-copy-id` in v0.5.0.
- Shipped Konsole reliability improvements and manual selected-server status checks in v0.6.0.
- Keep future release hardening focused on signing/notarization, cross-platform smoke testing, and packaging reliability.

## v0.7.0 Release Readiness

Status: in progress; includes the folded unreleased candidate and requires the v0.7.0 desktop lifecycle, automated gates, and mandatory AppImage acceptance before publication.

- Verify database integrity checks, pre-migration backups, and the v0.6.0 preservation fixture.
- Verify SSH destination rejection at persistence, import, and every OpenSSH launch path.
- Verify host terminal discovery, visible-selection testing, AppImage environment restoration, Konsole invocation, and every final launch outcome.
- Verify CSP, clipboard, and backend URL-opening boundaries.
- Verify migration 010 defaults/round trips, fixture upgrade and pre-010 backup, startup modes, tray fallbacks, explicit Quit, second-instance restoration, and fake-backed autostart outcomes.
- Require locked cargo checks on Ubuntu, Windows, and macOS; Windows and macOS remain unsigned and experimental.
- Require the generated AppImage to pass the full CachyOS/KDE Wayland acceptance gate before publishing the existing draft prerelease. Keep it draft when the required host or any mandatory result is unavailable.

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
