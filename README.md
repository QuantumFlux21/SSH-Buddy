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

The project is still pre-1.0. Expect UI polish, packaging work, and release automation changes before the first GitHub release.

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

## Development

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

Run frontend only:

```sh
npm run dev
```

Run the desktop app:

```sh
npm run tauri:dev
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

## Security Model

SSH-Buddy uses system OpenSSH and existing SSH keys. It stores key labels, key paths, optional fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or another user-controlled tool.

Normal terminal prompts remain the default for SSH passphrases, host key confirmation, passwords, and `sudo`. Automatic password injection and privileged command automation are intentionally out of scope.

Process execution is backend-owned. SSH launch builds argv arrays and launches a supported terminal without shell string interpolation. Web links are opened through the OS/browser opener after backend URL validation.

## Release Goal

The intended release model is one app and one codebase with platform-specific artifacts:

- Linux: AppImage first.
- Windows: NSIS `.exe` installer.
- macOS: `.dmg` for x64 and Apple Silicon.

GitHub Actions release automation is planned before the first public release.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
