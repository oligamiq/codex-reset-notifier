# Changelog

All notable changes to this project will be documented here.

## 0.1.1 - 2026-08-11

- Add checksum-verifying install scripts for Linux/macOS and Windows.
- Add one-line install commands to the README.
- Add installer syntax and smoke checks to CI.
- Publish installer scripts alongside release binaries.

## 0.1.0 - 2026-08-11

- Initial public release.
- Read Codex quota windows through the local Codex app-server.
- Push notifications through ntfy.
- Notify on quota exhaustion and quota reset transitions.
- Persist quota state across restarts.
- Use adaptive polling around known reset deadlines.
- Include a systemd user-service template for Linux.
