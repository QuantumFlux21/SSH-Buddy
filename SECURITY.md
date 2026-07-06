# Security Policy

## Supported Versions

`ssh-buddy` is pre-1.0. Security fixes target the latest main branch until release channels are established.

## Reporting a Vulnerability

Open a private security advisory on GitHub when the repository is public. Until then, avoid posting exploit details in public issues.

## Security Principles

- Use system OpenSSH instead of reimplementing SSH crypto.
- Do not store private key contents or passphrases.
- Do not store SSH, sudo, FTP, or RDP passwords.
- Do not inject passwords into SSH, sudo, or remote shells.
- Keep command execution narrowly scoped and backend-owned.
- Warn before risky features such as agent forwarding, root login, password storage, or automatic privileged commands.
