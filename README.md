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
- External terminal SFTP launch and copyable SFTP command generation through system OpenSSH.
- External RDP launch and copyable RDP command generation through FreeRDP.
- ProxyJump/bastion support through OpenSSH `-J`.
- Saved local SSH tunnel profiles with copy and external-terminal launch actions.
- Linux-first development flow, with CachyOS/KDE as the primary target environment.

The project is pre-1.0. `v0.1.0` is the first public GitHub release, and `v0.2.0` is the ProxyJump and local tunnel feature release.

## Stack

- Tauri 2 for the desktop shell.
- React, TypeScript, and Vite for the UI.
- Rust for the backend command layer.
- SQLite for local metadata persistence.
- System OpenSSH for SSH behavior.

## What SSH-Buddy Does Not Do

- Does not store private key contents, SSH passwords, passphrases, sudo passwords, or remote passwords.
- Does not automate sudo/root escalation.
- Does not implement SSH cryptography itself.
- Does not edit `~/.ssh/config`; import is preview-first and read-only.
- Does not implement FTP, FTPS, VNC, remote forwarding, SOCKS tunnels, embedded SFTP/RDP experiences, embedded terminals, sync, or KeePassXC integration yet.
- Does not promise one universal release file for every operating system.

## Install From GitHub Releases

Download release artifacts from:

https://github.com/QuantumFlux21/SSH-Buddy/releases

For Linux, download the AppImage, make it executable, and run it:

```sh
chmod +x SSH-Buddy-v0.2.0-linux-amd64.AppImage
./SSH-Buddy-v0.2.0-linux-amd64.AppImage
```

Replace `v0.2.0` with the version you downloaded if you are installing a different release.

The Linux AppImage expects the OpenSSH client and at least one supported external terminal in `PATH`: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm. Some distributions may also require AppImage/FUSE compatibility packages.

On some Wayland/WebKitGTK sessions, use the DMA-BUF workaround:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 ./SSH-Buddy-v0.2.0-linux-amd64.AppImage
```

For Windows, download the NSIS installer named like `SSH-Buddy-v0.2.0-windows-x64-setup.exe`.

For macOS, download the `.dmg` that matches your CPU:

- Apple Silicon: `SSH-Buddy-v0.2.0-darwin-aarch64.dmg`
- Intel: `SSH-Buddy-v0.2.0-darwin-x64.dmg`

Windows and macOS builds are currently unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Install From Source

Prerequisites:

- Node.js 22 or newer.
- Rust stable and Cargo.
- Linux packages required by Tauri/WebKitGTK for your distribution.
- OpenSSH client, including `ssh` and `sftp`.
- FreeRDP `xfreerdp3` or `xfreerdp` if you want RDP launch actions.
- At least one supported external terminal for SSH launch: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm.

On CachyOS, Arch, and Arch-based KDE systems, install the current Tauri Linux prerequisites from your package manager. Package names can change, but the needed pieces are Node.js/npm, Rust/Cargo, WebKitGTK, GTK, appindicator support, librsvg, and OpenSSH.

Install dependencies:

```sh
npm install
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
npm run build
npm test
cd src-tauri
cargo check
cargo test
```

## Local Data and Reset

SSH-Buddy stores local metadata in a SQLite database named `ssh-buddy.sqlite3` in the Tauri app data directory. The database stores profiles, groups, tags, SSH key path references, ProxyJump values, tunnel profiles, RDP settings, web links, notes, and settings. It does not store private key contents or passwords.

Typical locations are:

- Linux: `${XDG_DATA_HOME:-~/.local/share}/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- macOS: `~/Library/Application Support/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- Windows: `%APPDATA%\io.github.quantumflux21.ssh-buddy\ssh-buddy.sqlite3`

To reset local app data, close SSH-Buddy and delete the database file or the app data directory above. This removes SSH-Buddy's local metadata only; it does not delete SSH keys, edit `~/.ssh/config`, change `ssh-agent`, or touch remote servers.

## ProxyJump and Bastions

Server profiles can store an optional ProxyJump value. SSH-Buddy passes that value to OpenSSH as `-J` when launching SSH sessions or tunnel sessions. Supported examples include `bastion`, `user@bastion`, `user@bastion:22`, and comma-separated jump chains accepted by OpenSSH.

The SSH config import preview preserves detected `ProxyJump` values when selected candidates are imported. The preview still warns when a profile will use ProxyJump so the launch behavior is visible before import.

SSH-Buddy does not enable agent forwarding, store jump host passwords, or automate root/sudo workflows.

## SFTP External Launch

SSH-Buddy can launch the system OpenSSH `sftp` client in an external terminal for a saved server profile. SFTP uses the same host, username, identity file reference, ProxyJump value, and OpenSSH/ssh-agent setup as SSH.

For compatibility, SFTP commands use `-P <port>` for non-default ports, `-i <identity_file>` for selected key references, and `-o ProxyJump=<value>` for bastion hosts. SSH-Buddy does not store SFTP passwords or provide an embedded file browser yet; passphrase, password, and host-key prompts remain inside the external terminal/OpenSSH flow.

## RDP External Launch

SSH-Buddy can store per-server RDP launch settings and start FreeRDP externally using `xfreerdp3` when available, then `xfreerdp`. RDP settings can include username, domain, port, fullscreen, multi-monitor, optional monitor IDs such as `0,1`, dimensions, and color depth.

RDP commands are built from saved profile data only, for example `xfreerdp3 /v:host:3389 /u:username`. SSH-Buddy never stores or passes `/p:` password arguments. FreeRDP prompts interactively for credentials when needed.

## SSH Tunnels

SSH-Buddy supports saved local forwarding profiles per server. A tunnel profile stores a label, local bind host, local port, remote host, and remote port. The launch action runs OpenSSH in an external terminal using `ssh -N -L ...` and the selected server profile options, including port, identity file, and ProxyJump.

Tunnel sessions stay open only while the external terminal process is running. The default local bind host is `127.0.0.1`. Remote forwarding with `-R` and SOCKS forwarding with `-D` are not part of v0.2.0.

## Security Model

SSH-Buddy uses system OpenSSH and existing SSH keys. It stores key labels, key paths, optional fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or another user-controlled tool.

Normal terminal prompts remain the default for SSH passphrases, host key confirmation, passwords, and `sudo`. Automatic password injection and privileged command automation are intentionally out of scope.

Process execution is backend-owned. SSH, SFTP, RDP, and tunnel launch build argv arrays without shell string interpolation. SSH/SFTP/tunnel actions launch through supported terminals where appropriate; RDP launches the external FreeRDP client directly. ProxyJump, RDP settings, and tunnel values are validated before use. Web links are opened through the OS/browser opener after backend URL validation.

## Release Artifacts

The release model is one app and one codebase with platform-specific artifacts:

- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.

There is no single universal installer for every operating system.

Windows and macOS builds are unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Maintainer Release Process

1. For the v0.2.0 release, bump the version to `0.2.0` in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. The release workflow artifact names use the app version, so do not tag `v0.2.0` while these files still say `0.1.0`.
2. Run local checks:

```sh
npm run build
npm test
cd src-tauri
cargo check
cargo test
```

3. Commit the release-prep changes.
4. Push the tag:

```sh
git tag v0.2.0
git push origin v0.2.0
```

5. Review the draft GitHub release generated by `.github/workflows/release.yml`.
6. Confirm assets are present for Linux AppImage, Windows NSIS `.exe`, and macOS x64/Apple Silicon `.dmg`.
7. Confirm the release is marked as a pre-release if that is intended for the tag.
8. Smoke-test the Linux AppImage, including the Wayland workaround if needed.
9. Publish the draft after manual smoke testing.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
