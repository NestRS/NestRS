# NOTES.md — nestrs operational state

This file holds the **current** weather of the project — what works, what is
known-broken, what is queued. Unlike `CLAUDE.md` (durable rules), this file
moves: it is rewritten as items resolve. If you read it after a long pause,
treat every concrete claim (test counts, commands, file paths) as needing
verification before you act on it.

Update conventions:
- A resolved item is removed (not crossed out) — the git history is the
  changelog.
- Add a date in `YYYY-MM-DD` on an item that has been open for more than one
  cycle, so a future reader can judge whether the gap is fresh or stale.

## Verified baseline

Re-run after any authz, HTTP, or domain-data change:

```
NESTRS_DATABASE__URL=postgres://nestrs:nestrs@postgres:5432/nestrs cargo test -p api --test e2e
cargo test -p auth --test e2e
cargo test -p domain
```

At time of writing: `api/e2e` = 14 tests, `auth/e2e` = 10 tests, `domain`
covers authn aliases + authz policy + oauth grants.

## Open work — known gaps, not blockers for unrelated tasks

| Area | What |
|------|------|
| `nestrs-authz` (`http` feature) | Integration tests for `shape.rs` cover wire→mask→retain; expand to assert no `password_hash` leak in a domain-like scenario |
| `#[expose]` | Extend `wire.rs` default emission beyond `String`/`Option`/`bool`/numerics/`Uuid`/`DateTime*` — `Decimal`, custom enums, and other sea_orm types currently need a hand-written `impl WireModelDefaults` |
| `domain::users` | DB-backed tests for `users` `#[dataloader]` batches |
| `domain::oauth` | Unit tests for `OAuthFlow::resolve_caller` — needs `OAuth2Client` to become a trait (or a test-double) so the HTTP callout can be stubbed; `authenticate_client` likewise blocked on `OAuthFlow::new` requiring all four deps |
| Live check | Some PRs require `cargo run -p api` + `curl` + kill — see CONTRIBUTING |

## Feature exemplars — what to copy from when adding code

The repo holds a few canonical features; copy their shape before inventing
your own. When the exemplar no longer fits, fix it (and update this list).

- **`crates/domain/src/users/`** — the reference feature. `service.rs` holds
  `UsersService`, `CrudService`, `#[dataloader]`, credentials helpers in one
  file. `error.rs`: `UserError`, `CredentialError` (`Clone` where DataLoader
  needs it). `http.rs`: maps errors to HTTP in the domain crate
  (orphan-safe). `resolver.rs`: domain `UsersResolver` = relation `#[field]`
  only; `apps/api/src/users/resolver.rs` = root `#[query]` / `#[mutation]`.
  Batch loaders return `Result<HashMap<_, UserError>>`.
  `UsersService::new(db)` is public for tests. Custom `POST /users` returns
  `Json<User>` (DTO), not `Model`.
- **`crates/domain/src/oauth/`** — `service.rs` holds both `TokenIssuer`
  (signs claims into an `AccessToken`) and `OAuthFlow` (Authorization Code
  exchange, client-credentials validation in constant time). `strategy.rs`
  is a **thin HTTP adapter** over `OAuthFlow` (Poem request/response only —
  every grant decision lives in the service). `error.rs` + `http.rs` at the
  boundary. Grant logic tests in `domain/tests/oauth/`.
- **`crates/nestrs-authn/`** — the strict-mirror test layout reference:
  one `tests/authn.rs` entry, `tests/<role>/mod.rs` mirrors `src/<role>/`.
- **`apps/api/`** — the most complete app (REST + GraphQL + WS + DB + authz).
- **`apps/chat/`** — the pure real-time exemplar (WS-only, no DB).

