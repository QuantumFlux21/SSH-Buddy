# SSH-Buddy

SSH-Buddy is a Linux-first desktop SSH manager for homelab users. It organizes server profiles, groups, tags, SSH key references, notes, web admin links, and safe connection actions without becoming a general-purpose remote-access suite.

## Current Status

The core MVP is implemented:

- Local SQLite persistence in the Tauri app data directory.
- Server, group, tag, and SSH key reference CRUD.
- Web/admin links per server with `http://` and `https://` validation.
- `~/.ssh/config` import preview and selected import.
- External terminal SSH launch through system OpenSSH.
- Copyable SSH command generation.
- Public key install through system `ssh-copy-id` using saved SSH key references.
- External terminal SFTP launch and copyable SFTP command generation through system OpenSSH.
- External RDP launch and copyable RDP command generation through FreeRDP.
- ProxyJump/bastion support through OpenSSH `-J`.
- Saved local SSH tunnel profiles with copy and external-terminal launch actions.
- Manual server reachability checks with status lights, ping/TCP details, and selected-host port scan.
- Database integrity checks and recoverable backups before pending file migrations.
- Host terminal availability, visible-selection testing, AppImage environment restoration, and bounded immediate-exit diagnostics.
- Linux-first development flow, with CachyOS/KDE as the primary target environment.

The project is pre-1.0. The unreleased security, recovery, and launch-reliability candidate is being folded into `v0.7.0`; earlier releases introduced the core manager, tunnels, SFTP/RDP launch, clipboard diagnostics, RDP scaling, public key install, Konsole reliability, and selected-server status checks.

## Stack

- Tauri 2 for the desktop shell.
- React, TypeScript, and Vite for the UI.
- Rust for the backend command layer.
- SQLite for local metadata persistence.
- System OpenSSH for SSH behavior.

## What SSH-Buddy Does Not Do

- Does not store private key contents, SSH passwords, passphrases, sudo passwords, or remote passwords.
- Does not silently deploy SSH keys.
- Does not automate sudo/root escalation.
- Does not implement SSH cryptography itself.
- Does not edit `~/.ssh/config`; import is preview-first and read-only.
- Does not implement FTP, FTPS, VNC, remote forwarding, SOCKS tunnels, embedded SFTP/RDP experiences, embedded terminals, sync, background subnet scanning, or KeePassXC integration yet.
- Does not promise one universal release file for every operating system.

## Install From GitHub Releases

Download release artifacts from:

https://github.com/QuantumFlux21/SSH-Buddy/releases

For Linux, download the AppImage, make it executable, and run it:

```sh
chmod +x SSH-Buddy-v0.7.0-linux-amd64.AppImage
./SSH-Buddy-v0.7.0-linux-amd64.AppImage
```

Replace `v0.7.0` with the version you downloaded if you are installing a different release.

The Linux AppImage expects the OpenSSH client and at least one supported external terminal in `PATH`: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm. Some distributions may also require AppImage/FUSE compatibility packages.

On some Wayland/WebKitGTK sessions, use the DMA-BUF workaround:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 ./SSH-Buddy-v0.7.0-linux-amd64.AppImage
```

For Windows, download the NSIS installer named like `SSH-Buddy-v0.7.0-windows-x64-setup.exe`.

For macOS, download the `.dmg` that matches your CPU:

- Apple Silicon: `SSH-Buddy-v0.7.0-darwin-aarch64.dmg`
- Intel: `SSH-Buddy-v0.7.0-darwin-x64.dmg`

Windows and macOS builds are currently unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Install From Source

Prerequisites:

- Node.js 22 or newer.
- Rust stable and Cargo.
- Linux packages required by Tauri/WebKitGTK for your distribution.
- OpenSSH client, including `ssh`, `sftp`, and `ssh-copy-id` if you want public key install actions.
- FreeRDP `xfreerdp3` or `xfreerdp` if you want RDP launch actions.
- At least one supported external terminal for SSH launch: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm.

On CachyOS, Arch, and Arch-based KDE systems, install the current Tauri Linux prerequisites from your package manager. Package names can change, but the needed pieces are Node.js/npm, Rust/Cargo, WebKitGTK, GTK, appindicator support, librsvg, and OpenSSH.

Install dependencies:

```sh
npm ci
```

Build the frontend:

```sh
npm run build
```

Run the desktop app in development:

```sh
npm run tauri:dev
```

Build a local Tauri desktop bundle:

```sh
npm run tauri:build
```

Release AppImages should be built through the GitHub release workflow or a stable Ubuntu-style packaging environment. On rolling-release Linux systems, local AppImage packaging can fail inside `linuxdeploy` while stripping newer system libraries; this does not affect `npm run build`, tests, development runs, or the GitHub Actions release matrix.

## Development

Run frontend only:

```sh
npm run dev
```

On some Wayland sessions, WebKitGTK may need DMA-BUF rendering disabled:

```sh
npm run tauri:dev:linux
```

That script runs `WEBKIT_DISABLE_DMABUF_RENDERER=1 tauri dev`.

Run checks:

```sh
npm ci
npm run build
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
git diff --check
```

## Troubleshooting Launch and Clipboard Issues

If a copy button fails, SSH-Buddy now shows the command in a manual-copy panel. In the desktop app, clipboard writes use Tauri's clipboard manager plugin; in browser/Vite preview mode, SSH-Buddy falls back to the browser clipboard API when it is available.

If Open SSH, Open SFTP, public key install, tunnel launch, or RDP launch appears to do nothing, check the "Last launch attempt" panel. It shows the action type, selected terminal/client, resolved executable, command preview, key path checks, `.pub` file checks, required binary checks, host-environment restoration, and the final launch result: started, not found, pre-launch validation failed, spawn failed, or exited immediately. A started result confirms only the local terminal launch request, never an SSH or RDP connection.

On KDE/Wayland, the auto terminal path prefers Konsole when it is available and launches it with `konsole --separate --noclose -e <command> <args...>`. `--separate` avoids reusing an existing Konsole process, and `--noclose` keeps the window available when the child command exits. The diagnostics panel shows the exact resolved terminal argv and captures at most 2 KiB of sanitized stderr when a launcher exits during the immediate-failure window.

AppImage runtimes can prepend bundled paths and libraries that break host terminals. SSH-Buddy restores the original host path/library variables when available, removes AppImage-only runtime variables, and preserves desktop-session variables such as Wayland/X11, D-Bus, locale, and `SSH_AUTH_SOCK` before detecting or starting external tools.

If Konsole is unreliable on your session, set Settings -> Preferred terminal to Alacritty. Install Alacritty if needed, then retry Open SSH or Open SFTP. You can also manually run the copied SSH/SFTP command in any terminal.

Settings lists detected host terminals and their resolved executable paths. Use Test visible selection to launch a harmless fixed `printf` command with the value currently shown in the selector, even if it has not been saved. This verifies terminal detection and argv construction without connecting to a server.

For public key install, SSH-Buddy uses `ssh-copy-id` and the selected SSH key reference. The public key path is resolved as `<private-key-path>.pub`. If the `.pub` file is missing, create it with:

```sh
ssh-keygen -y -f ~/.ssh/id_ed25519 > ~/.ssh/id_ed25519.pub
```

If `ssh-copy-id` is missing, install the OpenSSH package that provides it for your distribution. On many Linux systems it is included with OpenSSH client packages. If installation prompts for a password, enter the remote server user's password in the external terminal; SSH-Buddy does not store it. If login still fails with "Permission denied" after install, verify the server username, `~/.ssh/authorized_keys` permissions, server `sshd_config` public-key settings, and any ProxyJump/bastion configuration.

For RDP, SSH-Buddy launches FreeRDP through the selected external terminal so certificate and password prompts have a usable TTY. If RDP closes immediately, copy the RDP command and run it manually from a terminal.

For high-DPI RDP sessions, configure RDP display/scaling on the server profile. The locally verified FreeRDP 3 options are native/default, `/scale:100`, `/scale:140`, `/scale:180`, `/smart-sizing`, and `+dynamic-resolution`. Start with `/scale:140` or `/scale:180` for high-DPI displays. Scaling support can vary by FreeRDP version, compositor, and desktop environment; if a mode behaves poorly, copy the RDP command and test it directly in a terminal.

If FreeRDP reports "Monitor configuration has gaps" or multi-monitor launch fails, first disable Multi-monitor and confirm a single-monitor session works. KDE monitor layouts used with `/multimon` may need contiguous monitor geometry. Then try explicit Monitor IDs in SSH-Buddy, for example `0,1`, and compare against:

```sh
xfreerdp3 /monitor-list
```

## Local Data and Reset

SSH-Buddy stores local metadata in a SQLite database named `ssh-buddy.sqlite3` in the Tauri app data directory. The database stores profiles, groups, tags, SSH key path references, ProxyJump values, tunnel profiles, RDP settings, web links, notes, and settings. It does not store private key contents or passwords.

Before applying pending migrations to an existing non-empty database, SSH-Buddy verifies database integrity and creates a sibling backup named like `ssh-buddy.sqlite3.pre-migration-<timestamp>-<id>.bak`. It verifies integrity again after migration. If open, integrity, backup, or migration fails, the app disables database access for that session and leaves the existing database in place. Preserve both the database and any backup before attempting recovery.

Typical locations are:

- Linux: `${XDG_DATA_HOME:-~/.local/share}/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- macOS: `~/Library/Application Support/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- Windows: `%APPDATA%\io.github.quantumflux21.ssh-buddy\ssh-buddy.sqlite3`

To reset local app data intentionally, close SSH-Buddy and delete the database file or the app data directory above. Do not use reset as a first response to a startup error; preserve the database and pre-migration backups for recovery. Reset removes SSH-Buddy's local metadata only; it does not delete SSH keys, edit `~/.ssh/config`, change `ssh-agent`, or touch remote servers.

## ProxyJump and Bastions

Server profiles can store an optional ProxyJump value. SSH-Buddy passes that value to OpenSSH as `-J` when launching SSH sessions or tunnel sessions. Supported examples include `bastion`, `user@bastion`, `user@bastion:22`, and comma-separated jump chains accepted by OpenSSH.

The SSH config import preview preserves detected `ProxyJump` values when selected candidates are imported. The preview still warns when a profile will use ProxyJump so the launch behavior is visible before import.

SSH-Buddy does not enable agent forwarding, store jump host passwords, or automate root/sudo workflows.

## Server Status and Port Scan

SSH-Buddy can manually check reachability for the selected server. The left server list shows a status dot after a check has run:

- Unknown: no check has run yet.
- Online: ping and the primary TCP port both respond.
- Degraded: ping works but the primary TCP port fails, or ping fails/is unavailable while TCP works.
- Offline: both ping and the primary TCP port fail.
- Checking: a manual check is currently running.

The primary TCP port is the SSH port for normal SSH profiles. If RDP is enabled for the selected profile, the RDP port is used as the primary service port. Ping can be blocked by firewalls or unavailable in sandboxed environments, so SSH-Buddy always performs the TCP connect check as well.

The Scan ports button is manual and selected-host only. It checks a small allowlist: 22, 80, 443, 3389, 5432, 6379, 8080, and 8443. SSH-Buddy does not scan subnets, discover networks, run background polling, or store monitoring history. Use port scan only on servers you own or administer.

## SFTP External Launch

SSH-Buddy can launch the system OpenSSH `sftp` client in an external terminal for a saved server profile. SFTP uses the same host, username, identity file reference, ProxyJump value, and OpenSSH/ssh-agent setup as SSH.

For compatibility, SFTP commands use `-P <port>` for non-default ports, `-i <identity_file>` for selected key references, and `-o ProxyJump=<value>` for bastion hosts. SSH-Buddy does not store SFTP passwords or provide an embedded file browser yet; passphrase, password, and host-key prompts remain inside the external terminal/OpenSSH flow.

## Install Public Key

SSH-Buddy can install the matching public key for a saved SSH key reference on a selected server profile. The action uses system `ssh-copy-id` in the selected external terminal so remote password and host-key prompts stay interactive.

SSH-Buddy stores the key path only. It does not import, copy, read, or store private key contents. For a key reference path such as `~/.ssh/id_ed25519_homelab`, SSH-Buddy resolves the public key path as `~/.ssh/id_ed25519_homelab.pub` and builds a command like:

```sh
ssh-copy-id -i ~/.ssh/id_ed25519_homelab.pub -p 22 user@host
```

If the server profile uses ProxyJump, SSH-Buddy passes it as a validated OpenSSH option:

```sh
ssh-copy-id -i ~/.ssh/id_ed25519_homelab.pub -p 22 -o ProxyJump=bastion user@host
```

The install action requires explicit confirmation before launch. SSH-Buddy does not store server passwords, SSH key passphrases, or sudo passwords, and it does not modify sudoers.

## RDP External Launch

SSH-Buddy can store per-server RDP launch settings and start FreeRDP externally using `xfreerdp3` when available, then `xfreerdp`. Launch uses the selected external terminal so FreeRDP can prompt for certificate trust or credentials. RDP settings can include username, domain, port, certificate mode, fullscreen, multi-monitor, optional monitor IDs such as `0,1`, dimensions, color depth, and display/scaling mode.

RDP commands are built from saved profile data only, for example `xfreerdp3 /v:host:3389 /cert:tofu /scale:140 /u:username`. Certificate mode can be default/prompt, trust on first use with `/cert:tofu`, or ignore with `/cert:ignore`. Trust on first use is recommended for many Windows RDP hosts with self-signed certificates. Ignore is less secure and is never selected silently. Display/scaling modes are allowlisted to native/default, `/scale:100`, `/scale:140`, `/scale:180`, `/smart-sizing`, and `+dynamic-resolution`. With multi-monitor enabled, SSH-Buddy passes `/multimon`; if monitor IDs are configured, it passes a validated value such as `/monitors:0,1`. SSH-Buddy never stores or passes `/p:` password arguments. FreeRDP prompts interactively for credentials when needed.

## SSH Tunnels

SSH-Buddy supports saved local forwarding profiles per server. A tunnel profile stores a label, local bind host, local port, remote host, and remote port. The launch action runs OpenSSH in an external terminal using `ssh -N -L ...` and the selected server profile options, including port, identity file, and ProxyJump.

Tunnel sessions stay open only while the external terminal process is running. The default local bind host is `127.0.0.1`. Remote forwarding with `-R` and SOCKS forwarding with `-D` are not implemented yet.

## Security Model

SSH-Buddy uses system OpenSSH and existing SSH keys. It stores key labels, key paths, optional fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or another user-controlled tool.

Normal terminal prompts remain the default for SSH passphrases, host key confirmation, passwords, public key install via `ssh-copy-id`, and `sudo`. Automatic password injection and privileged command automation are intentionally out of scope.

Process execution is backend-owned. SSH, SFTP, public key install, RDP, tunnel launch, terminal tests, ping checks, and selected-host port checks build argv/process calls without shell string interpolation. SSH destinations are validated before persistence, SSH config resolution/import, command display, and launch; option-like values, whitespace/control characters, and ambiguous `@` use are rejected while aliases and IPv4/IPv6 forms remain supported. ProxyJump, RDP settings, and tunnel values are validated before use. Web links are opened through the OS/browser opener after backend URL validation.

The desktop content security policy permits local application resources and required Tauri IPC only; development additionally permits the local Vite WebSocket. Clipboard capability is write-only. Frontend link clicks do not receive opener permission, and saved web links remain backend-opened after validation.

## Release Artifacts

The release model is one app and one codebase with platform-specific artifacts:

- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.

There is no single universal installer for every operating system.

Windows and macOS builds are unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Maintainer Release Process

1. Confirm the version is `0.7.0` in `package.json`, the root entries in `package-lock.json`, `src-tauri/Cargo.toml`, the `ssh-buddy` entry in `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
2. Run local checks:

```sh
npm ci
npm run build
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
git diff --check
```

3. Publish the release branch and open a pull request against `main`; wait for CI and review before merge.
4. After separate approval to merge, merge the pull request without rewriting shared history.
5. After separate tag approval, create and push the tag:

```sh
git tag v0.7.0
git push origin v0.7.0
```

6. Review the draft prerelease generated by `.github/workflows/release.yml`.
7. Verify the Linux AppImage, Windows NSIS `.exe`, macOS x64 `.dmg`, and macOS Apple Silicon `.dmg`; smoke-test available platforms, including AppImage database upgrade and KDE/Konsole behavior.
8. Obtain separate publication approval, then publish the draft. Windows and macOS artifacts remain unsigned.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
