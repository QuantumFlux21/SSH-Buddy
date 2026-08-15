# Contributing

Thanks for helping with `ssh-buddy`.

## Local Checks

Before opening a pull request:

```sh
npm ci
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
git diff --check
```

## Optional isolated check harness

`Dockerfile.vm-test` and `compose.vm-test.yml` provide non-root, runtime-network-disabled frontend and Rust environments for the same release checks. They are intended for a trusted disposable runner when the host does not have the Tauri system libraries or Rust toolchain installed.

```sh
docker compose -f compose.vm-test.yml build frontend rust
docker compose -f compose.vm-test.yml run --rm frontend npm run build
docker compose -f compose.vm-test.yml run --rm frontend npm test
docker compose -f compose.vm-test.yml run --rm rust cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
docker compose -f compose.vm-test.yml run --rm rust cargo check --locked --manifest-path src-tauri/Cargo.toml
docker compose -f compose.vm-test.yml run --rm rust cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Image builds perform the lockfile installs (`npm ci` and `cargo fetch --locked`) before runtime networking is disabled. The harness does not package or publish release artifacts.

## Scope

SSH-Buddy is an SSH-focused manager. Its current scope includes external SSH and SFTP launch, public key install through `ssh-copy-id`, external RDP launch, saved local SSH tunnels, web links, and manual selected-server reachability checks.

Keep FTP/FTPS, VNC, SCP helpers, embedded SFTP/RDP or terminal experiences, remote (`-R`) and SOCKS (`-D`) forwarding, Wake-on-LAN, custom command snippets, credential-manager integration, sync, and broad or background network scanning behind explicit roadmap discussion.

## Security-sensitive Changes

Changes involving process execution, SSH config import, key handling, external URLs, terminals, or privileged workflows need tests and a short security note in the pull request.

Desktop lifecycle changes also require pure fake-based tests. Automated tests must not modify the runner's real login startup registration. Keep window, tray, autostart, and single-instance operations backend-owned; do not add frontend tray/autostart capabilities.

Before release, require the local checks above plus locked `cargo check` jobs on Ubuntu, Windows, and macOS. Linux-only Tauri development packages belong only in the Ubuntu jobs. The generated AppImage must pass the documented CachyOS/KDE Wayland acceptance gate before the existing draft prerelease is published.
