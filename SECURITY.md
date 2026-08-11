# Security

## Reporting a vulnerability

Please do not open a public issue for a security vulnerability. Use GitHub's private vulnerability reporting for this repository when available, or contact the repository owner privately through GitHub.

## Sensitive data

This program does not read or persist OpenAI credentials itself. It launches the locally installed Codex app-server and relies on Codex's existing authentication.

An ntfy topic can act like a shared secret when using the public ntfy service. Use a long random topic, do not commit it to the repository, and use an authenticated/self-hosted ntfy server when stronger access control is required.
