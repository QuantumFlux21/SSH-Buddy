# SSH-Buddy

SSH-Buddy is a Linux-first desktop SSH manager for homelab users. It is designed to organize servers, keys, tags, notes, and safe connection actions without becoming a general-purpose remote-access suite.

## Current Status

The project foundation is in place:

- Tauri 2 desktop scaffold.
- React and TypeScript UI shell.
- Rust entrypoint and Tauri command contract.
- Linux development workflow.

The app currently uses minimal in-memory backend behavior so the desktop shell can run. SQLite persistence, real SSH config import, external terminal launching, and browser opening are planned MVP work and are not implemented yet.

## MVP Direction

- Manage SSH server profiles with hostname/IP, port, username, identity file path, group, tags, notes, and web admin links.
- Import concrete host entries from `~/.ssh/config` through a preview flow.
- Launch SSH sessions in an external terminal.
- Copy generated SSH commands.
- Open server web admin URLs.
- Store key references and fingerprints, not private key contents.

Not in the MVP: FTP, RDP, VNC, SCP helpers, SFTP browser, embedded file transfer, tunnels, Wake-on-LAN, password storage, and automatic privileged commands.

## Stack

- Tauri 2 for the desktop shell.
- React, TypeScript, and Vite for the UI.
- Rust for the backend command layer.
- SQLite is planned for local metadata persistence.
- System OpenSSH for SSH behavior.

## Development

Prerequisites:

- Node.js 22 or newer.
- Rust stable and Cargo.
- Linux packages required by Tauri/WebKitGTK for your distribution.
- OpenSSH client.

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
npm test
npm run build
cd src-tauri && cargo check
```

## Security Model

SSH-Buddy uses system OpenSSH and existing SSH keys. It does not store raw private keys, SSH passphrases, sudo passwords, RDP passwords, or FTP passwords in its database. Key records are references to files already managed by the OS, OpenSSH, `ssh-agent`, or another user-controlled tool.

Normal terminal prompts remain the default for SSH passphrases, host key confirmation, passwords, and `sudo`. Automatic password injection and broad privileged command automation are intentionally out of scope.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
