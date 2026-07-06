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

The app should store references to key files, labels, public fingerprints, and profile metadata only. Private key contents stay in user-controlled OpenSSH files, the OS, `ssh-agent`, or a dedicated password manager chosen by the user.

## Privileged Workflows

SSH-Buddy must not automate sudo by storing or injecting sudo passwords. Normal interactive sudo prompts remain the default. Any future automation around privileged commands must require explicit user configuration, clear warnings, and narrow command scope.
