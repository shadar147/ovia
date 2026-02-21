# Ovia — Microtasks (5–10 min chunks)

Rule: each task should be completable in one focused sprint (<=10 min), with a clear output artifact.

## OVIA-0002 CI quality gates (decomposed)
- [ ] MT-0002-01 Create `.github/workflows/ci.yml` skeleton with trigger on push/PR.
- [ ] MT-0002-02 Add rust toolchain setup step.
- [ ] MT-0002-03 Add `cargo fmt --all --check` step.
- [ ] MT-0002-04 Add `cargo clippy --all-targets --all-features -- -D warnings` step.
- [ ] MT-0002-05 Add `cargo test --all` step.
- [ ] MT-0002-06 Add artifact upload for test logs.
- [ ] MT-0002-07 Add README badge/link to CI status.
- [ ] MT-0002-08 Validate workflow syntax locally with `act`-compatible lint (or `yamllint` if available).
- [ ] MT-0002-09 Create `docs/.planning/WO-0002-review.md` template.

## OVIA-1001 SQL migration baseline (decomposed)
- [ ] MT-1001-01 Add index for `person_identity_links(status, confidence)`.
- [ ] MT-1001-02 Add index for `identities(org_id, source, email)` where email is not null.
- [ ] MT-1001-03 Add index for `identities(org_id, source, username)` where username is not null.
- [ ] MT-1001-04 Add comments on `status` semantics in SQL.
- [ ] MT-1001-05 Add query example for conflict queue in docs.
- [ ] MT-1001-06 Add migration apply check instructions in docs.

## OVIA-1002 Identity repository layer (decomposed)
- [ ] MT-1002-01 Define repository trait file for `people`.
- [ ] MT-1002-02 Define repository trait file for `identities`.
- [ ] MT-1002-03 Define repository trait file for `person_identity_links`.
- [ ] MT-1002-04 Define repository trait file for `identity_events`.
- [ ] MT-1002-05 Add DTOs for list/filter requests.
- [ ] MT-1002-06 Implement `list_mappings` query (read-only).
- [x] MT-1002-07 Add unit test fixture for mapping list.
- [x] MT-1002-08 Implement `confirm_mapping` transaction.
- [x] MT-1002-09 Add integration test for confirm mapping.
- [x] MT-1002-10 Implement `remap_mapping` transaction.
- [x] MT-1002-11 Add integration test for remap.
- [x] MT-1002-12 Implement `split_mapping` transaction.
- [x] MT-1002-13 Add integration test for split.

## OVIA-1003 Identity API v1 (decomposed)
- [x] MT-1003-01 Add route module for identity mappings.
- [x] MT-1003-02 Implement `GET /team/identity-mappings` handler.
- [x] MT-1003-03 Implement query validation for filters.
- [x] MT-1003-04 Add response schema structs.
- [x] MT-1003-05 Add `POST /confirm` handler.
- [x] MT-1003-06 Add `POST /remap` handler.
- [x] MT-1003-07 Add `POST /split` handler.
- [x] MT-1003-08 Emit audit event on each mutation.
- [x] MT-1003-09 Add handler tests for error mapping.

## OVIA-2001 Matching rules v1 (decomposed)
- [x] MT-2001-01 Define scoring config struct + defaults.
- [x] MT-2001-02 Implement exact-email scorer.
- [x] MT-2001-03 Implement username-similarity scorer.
- [x] MT-2001-04 Implement display-name scorer.
- [x] MT-2001-05 Implement team/project co-occurrence scorer.
- [x] MT-2001-06 Implement service-account penalty/exclusion.
- [x] MT-2001-07 Merge scorers into final confidence function.
- [x] MT-2001-08 Add rule-trace payload generation.
- [x] MT-2001-09 Add threshold classifier (`auto/conflict/reject`).
- [x] MT-2001-10 Add fixture tests (at least 15 scenarios).

## OVIA-2002 Conflict queue workflow (decomposed)
- [x] MT-2002-01 Add conflict queue query endpoint.
- [x] MT-2002-02 Add sort options (confidence asc, age desc).
- [x] MT-2002-03 Add bulk confirm endpoint.
- [x] MT-2002-04 Add CSV export formatter.
- [x] MT-2002-05 Add metrics counters for queue size.

## OVIA-5001 Swarm stack manifests (decomposed)
- [x] MT-5001-01 Create multi-stage `backend/Dockerfile` (builder + runtime).
- [x] MT-5001-02 Create `backend/.dockerignore`.
- [x] MT-5001-03 Create `backend/infra/docker-compose.swarm.yml` with all services.
- [x] MT-5001-04 Create `backend/infra/Caddyfile` for reverse proxy.
- [x] MT-5001-05 Create `backend/infra/init-db.sh` migration script.
- [x] MT-5001-06 Create `.env.example` with production variable template.
- [x] MT-5001-07 Update delivery backlog status to done.

## Operating cadence
- One commit every 1–2 microtasks (max ~10 minutes work).
- Each commit includes:
  - What changed
  - Checks run
  - Next microtask
