# Contributing

Thanks for helping with `ssh-buddy`.

## Local Checks

Before opening a pull request:

```sh
npm test
npm run build
cd src-tauri && cargo test
```

## Scope

The MVP is an SSH manager. Keep FTP, RDP, VNC, SCP helpers, SFTP browser, tunnels, Wake-on-LAN, and custom command snippets behind explicit roadmap discussion.

## Security-sensitive Changes

Changes involving process execution, SSH config import, key handling, external URLs, terminals, or privileged workflows need tests and a short security note in the pull request.
