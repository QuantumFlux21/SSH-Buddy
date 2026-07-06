# Security Policy

## Supported Versions

SSH-Buddy is pre-1.0. Security fixes target the latest main branch until release channels are established.

## Reporting a Vulnerability

Open a private security advisory on GitHub when the repository is public. Until then, avoid posting exploit details in public issues.

## Security Principles

- Use system OpenSSH instead of reimplementing SSH crypto.
- Prefer existing OpenSSH key files and `ssh-agent`.
- Do not store private key contents in the app database.
- Do not store SSH passphrases or SSH passwords.
- Do not store sudo passwords.
- Do not store RDP passwords.
- Do not inject passwords into SSH, sudo, RDP, or remote shells.
- Do not store FTP, RDP, or other remote-access passwords.
- Keep command execution narrowly scoped and backend-owned.
- Warn before risky features such as agent forwarding, root login, password storage, or automatic privileged commands.

## SSH Key Handling

The app stores references to key files, labels, optional public fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or a dedicated password manager chosen by the user.

## Privileged Workflows

SSH-Buddy must not automate sudo by storing or injecting sudo passwords. Normal interactive sudo prompts remain the default. Any future automation around privileged commands must require explicit user configuration, clear warnings, and narrow command scope.

## Current Release Guarantees

- SSH launch uses the system `ssh` binary and argv/process APIs.
- SSH, SFTP, RDP, and tunnel launch use argv/process APIs and do not pass commands through a shell.
- SSH config import reads `~/.ssh/config` but does not create, edit, or overwrite it.
- Import stores explicit `IdentityFile` paths as key references only.
- ProxyJump values are stored as local metadata and passed to OpenSSH with `-J` after validation.
- SFTP launch uses the system OpenSSH `sftp` client and passes ProxyJump with `-o ProxyJump=...` for compatibility.
- RDP launch uses `xfreerdp3` or `xfreerdp` and never includes `/p:` password arguments.
- RDP monitor selection is passed only as validated `/monitors:<ids>` values.
- Local SSH tunnels are launched with OpenSSH `-N -L` after validating bind host, remote host, and ports.
- Web/admin links must be `http://` or `https://` and must not contain embedded credentials.
- Notes are plaintext local metadata and should not contain secrets.

## ProxyJump and Tunnel Safety

ProxyJump support delegates bastion behavior to OpenSSH. SSH-Buddy stores the `ProxyJump` host spec only, validates it before launch, and passes it as an argv value to `ssh -J`. It does not store jump host passwords, enable agent forwarding, or automate privileged access.

SSH tunnel support is limited to local forwarding for now. Tunnel launch uses `ssh -N -L local_bind_host:local_port:remote_host:remote_port` in an external terminal. The default local bind host is `127.0.0.1`; binding to broader interfaces can expose forwarded services to other machines and should be used only when intended.

## SFTP Safety

SFTP support delegates file-transfer behavior to the system OpenSSH `sftp` client in an external terminal. SSH-Buddy builds argv values from saved profile metadata only. It does not store SFTP passwords, read private key contents, or inject credentials. Any passphrase, password, or host-key prompt remains part of the normal OpenSSH terminal flow.

## RDP Safety

RDP support delegates remote desktop behavior to FreeRDP through `xfreerdp3` or `xfreerdp`. SSH-Buddy stores only launch metadata such as username, domain, port, display mode, monitor IDs, dimensions, and color depth. Monitor IDs are validated as comma-separated monitor numbers and are not an arbitrary FreeRDP option field. SSH-Buddy does not store RDP passwords and does not pass `/p:` arguments. FreeRDP prompts interactively when credentials are required.

RDP settings are separate from SSH/SFTP assumptions. SSH keys, ssh-agent, ProxyJump, and sudo/root workflows do not authenticate RDP.

## Local Metadata Storage

SSH-Buddy stores local app metadata in `ssh-buddy.sqlite3` under the Tauri app data directory for the app identifier `io.github.quantumflux21.ssh-buddy`. This database is not an encrypted secret store. Delete the app data directory to reset local profiles, groups, tags, key references, ProxyJump values, tunnel profiles, RDP settings, web links, notes, and settings.

## What SSH-Buddy Does Not Do

- Does not store private key contents.
- Does not store SSH passwords or passphrases.
- Does not store SFTP passwords.
- Does not store RDP passwords.
- Does not store sudo passwords.
- Does not inject passwords into SSH, sudo, remote shells, RDP, FTP, or web URLs.
- Does not automate root login or privileged command execution.
- Does not enable agent forwarding by default.
- Does not implement FTP, FTPS, remote forwarding, SOCKS tunnels, embedded SFTP/RDP experiences, embedded terminal sessions, sync, or KeePassXC integration.
