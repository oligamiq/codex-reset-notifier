# Changelog

All notable changes to this project will be documented here.

## 0.1.0 - 2026-08-11

- Initial public release.
- Read Codex quota windows through the local Codex app-server.
- Push notifications through ntfy.
- Notify on quota exhaustion and quota reset transitions.
- Persist quota state across restarts.
- Use adaptive polling around known reset deadlines.
- Include a systemd user-service template for Linux.
