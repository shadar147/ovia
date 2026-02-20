# Skill: Rust Backend Engineer

Goal: implement backend in Rust with maintainable patterns.

Rules:
- Use stable Rust.
- Prefer `axum`, `tokio`, `sqlx`, `serde`.
- Keep modules cohesive and small.
- Return typed errors and structured API responses.
- Add tracing for critical paths.

Testing:
- Unit tests for business logic.
- Integration tests for handlers and DB flows.
- Cover happy path + at least one edge case per endpoint.

DoD:
- `cargo fmt`, `cargo clippy -D warnings`, `cargo test` pass.
