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
- Do not inject passwords into SSH, sudo, or remote shells.
- Do not store FTP, RDP, or other remote-access passwords by default if those integrations are added later.
- Keep command execution narrowly scoped and backend-owned.
- Warn before risky features such as agent forwarding, root login, password storage, or automatic privileged commands.

## SSH Key Handling

The app stores references to key files, labels, optional public fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or a dedicated password manager chosen by the user.

## Privileged Workflows

SSH-Buddy must not automate sudo by storing or injecting sudo passwords. Normal interactive sudo prompts remain the default. Any future automation around privileged commands must require explicit user configuration, clear warnings, and narrow command scope.

## Current MVP Guarantees

- SSH launch uses the system `ssh` binary and argv/process APIs.
- SSH-Buddy does not pass SSH commands through a shell for launch.
- SSH config import reads `~/.ssh/config` but does not create, edit, or overwrite it.
- Import stores explicit `IdentityFile` paths as key references only.
- Web/admin links must be `http://` or `https://` and must not contain embedded credentials.
- Notes are plaintext local metadata and should not contain secrets.

## Local Metadata Storage

SSH-Buddy stores local app metadata in `ssh-buddy.sqlite3` under the Tauri app data directory for the app identifier `io.github.quantumflux21.ssh-buddy`. This database is not an encrypted secret store. Delete the app data directory to reset local profiles, groups, tags, key references, web links, notes, and settings.

## What SSH-Buddy Does Not Do

- Does not store private key contents.
- Does not store SSH passwords or passphrases.
- Does not store sudo passwords.
- Does not inject passwords into SSH, sudo, remote shells, RDP, FTP, or web URLs.
- Does not automate root login or privileged command execution.
- Does not enable agent forwarding by default.
- Does not implement SFTP, RDP, tunnels, embedded terminal sessions, sync, or KeePassXC integration in the MVP.
