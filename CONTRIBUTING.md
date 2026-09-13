# contributing to mew

mew is a minimal, sub-10ms terminal companion and git workspace status dashboard.

## development loop

all pull requests must pass formatting, linter, and unit test checks cleanly:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## guidelines

- all code comments, documentation, and user-facing messages must remain lowercase.
- do not add unicode emoji to cli text, markdown, or status output.
- use conventional commit prefixes (`feat:`, `fix:`, `docs:`, `perf:`, `refactor:`, `test:`, `chore:`).
- no fake telemetry: every number displayed must come from a real source or honestly indicate `not available`.
