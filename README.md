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
- Linux-first development flow, with CachyOS/KDE as the primary target environment.

The project is pre-1.0. `v0.1.0` is the first public GitHub release; a `v0.1.1` release should stay limited to bug fixes, documentation, and packaging corrections unless a new milestone is approved.

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
- Does not implement SFTP, RDP, VNC, tunnels, embedded terminals, sync, or KeePassXC integration yet.
- Does not promise one universal release file for every operating system.

## Install From GitHub Releases

Download release artifacts from:

https://github.com/QuantumFlux21/SSH-Buddy/releases

For Linux, download the AppImage, make it executable, and run it:

```sh
chmod +x SSH-Buddy-v0.1.0-linux-amd64.AppImage
./SSH-Buddy-v0.1.0-linux-amd64.AppImage
```

The Linux AppImage expects the OpenSSH client and at least one supported external terminal in `PATH`: Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm. Some distributions may also require AppImage/FUSE compatibility packages.

On some Wayland/WebKitGTK sessions, use the DMA-BUF workaround:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 ./SSH-Buddy-v0.1.0-linux-amd64.AppImage
```

For Windows, download the NSIS installer named like `SSH-Buddy-v0.1.0-windows-x64-setup.exe`.

For macOS, download the `.dmg` that matches your CPU:

- Apple Silicon: `SSH-Buddy-v0.1.0-darwin-aarch64.dmg`
- Intel: `SSH-Buddy-v0.1.0-darwin-x64.dmg`

Windows and macOS builds are currently unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Install From Source

Prerequisites:

- Node.js 22 or newer.
- Rust stable and Cargo.
- Linux packages required by Tauri/WebKitGTK for your distribution.
- OpenSSH client.
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

SSH-Buddy stores local metadata in a SQLite database named `ssh-buddy.sqlite3` in the Tauri app data directory. The database stores profiles, groups, tags, SSH key path references, web links, notes, and settings. It does not store private key contents or passwords.

Typical locations are:

- Linux: `${XDG_DATA_HOME:-~/.local/share}/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- macOS: `~/Library/Application Support/io.github.quantumflux21.ssh-buddy/ssh-buddy.sqlite3`
- Windows: `%APPDATA%\io.github.quantumflux21.ssh-buddy\ssh-buddy.sqlite3`

To reset local app data, close SSH-Buddy and delete the database file or the app data directory above. This removes SSH-Buddy's local metadata only; it does not delete SSH keys, edit `~/.ssh/config`, change `ssh-agent`, or touch remote servers.

## Security Model

SSH-Buddy uses system OpenSSH and existing SSH keys. It stores key labels, key paths, optional fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or another user-controlled tool.

Normal terminal prompts remain the default for SSH passphrases, host key confirmation, passwords, and `sudo`. Automatic password injection and privileged command automation are intentionally out of scope.

Process execution is backend-owned. SSH launch builds argv arrays and launches a supported terminal without shell string interpolation. Web links are opened through the OS/browser opener after backend URL validation.

## Release Artifacts

The release model is one app and one codebase with platform-specific artifacts:

- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.

There is no single universal installer for every operating system.

Windows and macOS pre-release builds are unsigned. Windows SmartScreen, macOS Gatekeeper, and browser download warnings may appear. Proper Windows code signing and macOS signing/notarization are future release-hardening work.

## Maintainer Release Process

1. Confirm the version is `0.1.0` in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
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
git tag v0.1.0
git push origin v0.1.0
```

5. Review the draft GitHub release generated by `.github/workflows/release.yml`.
6. Confirm assets are present for Linux AppImage, Windows NSIS `.exe`, and macOS x64/Apple Silicon `.dmg`.
7. Confirm the release is marked as a pre-release if that is intended for the tag.
8. Smoke-test the Linux AppImage, including the Wayland workaround if needed.
9. Publish the draft after manual smoke testing.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
