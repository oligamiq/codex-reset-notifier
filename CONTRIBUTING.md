# Contributing

Contributions are welcome.

Before submitting a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

Please keep changes focused and include tests for quota transition logic when behavior changes.

This project depends on behavior exposed by the Codex app-server. If a Codex update changes the rate-limit response shape, include the Codex CLI version and a sanitized example of the changed response in the issue or pull request.
