# Assessment: `eventually-rs`

## Executive Summary

`eventually-rs` is a well-intentioned DDD/Event Sourcing toolkit with a thoughtful module layout and a solid core model. However, the repo is currently in a **broken state** on the default branch and several design choices leak awkwardness into user code. Release automation is absent, and maintenance signals (branches left dangling, stale `wip` commit on `main`, `feat/public-crate-api` checked-out branch) suggest the project stalled mid‑refactor.

The good news: the bones are right. The model — `Aggregate` + `Root<T>` + `event::Store` + `aggregate::Repository` + test `Scenario` — closely mirrors proven Go libraries (eventide, goengine) and EventStore patterns. With a focused round of cleanup, this could become a credible canonical Rust ES crate.

---

## 1. Build & correctness — Critical

Current HEAD (branch `feat/public-crate-api`, commit `7a4cd2a wip`) **does not compile**:

- `eventually-macros/src/lib.rs:45,52,65,66,71` reference `__eventually_crate::aggregate::Root` with no `extern crate` alias or re-export anywhere in the workspace. `cargo build --workspace --all-targets` fails with `E0433: failed to resolve: use of unresolved module or unlinked crate __eventually_crate` for both the `bank-accounting` example and `eventually-postgres` integration tests.
- Even if that is fixed, the `aggregate_root` attribute produces `impl Deref<Target = Root<T>>` but user code (`examples/bank-accounting/src/domain.rs:201,221,231,235,254,258,277,286,294`, `eventually-postgres/tests/setup/mod.rs:103,105`) accesses inner Aggregate fields directly (`self.is_closed`, `self.id`, `self.pending_transactions`). Field autoderef does not chain through two `Deref` hops in Rust, and `BankAccount`'s fields are private anyway, so the example's `self.is_closed` is unreachable via `Deref` even with a single hop. The macro and the intended usage pattern are fundamentally misaligned.
- `main` itself has a `wip` commit on top (`7a4cd2a`), which is a smell. CI on main: should tell whether that state has been pushed.

**Fix options:**
- Have `eventually` re-export under a stable known path (`pub use eventually_core as __eventually_crate;`) and document that users must depend on `eventually` (not `eventually-core`) when using the macro. Or, have the macro emit `::eventually::aggregate::Root<...>` and document dependency constraints; or detect the crate via `proc-macro-crate`.
- Rework the macro to also provide direct field accessors (e.g. `self.aggregate()` / `self.aggregate_mut()`) or generate a `Deref<Target = Aggregate>` chain — and reconsider whether forcing private Aggregate fields is the right default.

## 2. API design

### What works well
- Clear separation between `Aggregate` (pure domain) and `Root<T>` (transactional/state envelope). `record_new` / `record_that` force events through the state transition, and `take_uncommitted_events` is appropriately hidden (`#[doc(hidden)]`).
- The `Streamer` + `Appender` split in `event::store.rs` with the auto-implemented `Store` marker trait is clean and lets backends implement only what they need.
- `Tracking<T>` decorator and the `Scenario` given/when/then helpers provide a nice testing story.
- `version::Check` / `version::ConflictError` model is minimal and correct for optimistic concurrency.
- `Metadata = HashMap<String, String>` keeps the wire story simple and plays well with JSON/Protobuf.

### Rough edges

1. **`Version = u64`** (`eventually-core/src/version.rs:7`) is a type alias, not a newtype. Throughout `eventually-postgres` this forces `as i32` casts with `#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]` sprinkled everywhere. A newtype wrapping `u64` (or the richer Postgres `i64`) would avoid the cast storm and make "expected + len" math safer.

2. **`Aggregate::apply(state: Option<Self>, event) -> Result<Self, _>`** takes the state by value and returns a new state. Fine for small aggregates, but forces `Clone` on every `record_that` (`Root::record_that` does `T::apply(Some(self.aggregate.clone()), ...)`). Consider `&mut Self` or a pair (`apply_first`, `apply`) to avoid the clone, or make the `Clone` bound explicit at `Aggregate`.

3. **Two `aggregate_id` methods.** `Aggregate::aggregate_id(&self)` and `Root::aggregate_id(&self)` do the same thing by delegation. Minor, but confusing.

4. **`Root::to_aggregate_type<K: From<T>>`** requires `Clone` internally and returns `K` — but `From<T>` is the wrong bound for most cases because users typically need a `&T` (to serialize). It's effectively doing `K::from(self.aggregate.clone())`. Either document it is "pay to project the state into a persistence DTO" or add a `&self → &T` helper.

5. **`Root<T>: Deref<Target = T>` is provided** (`aggregate/mod.rs:137`) — so consumers can already read the state. But the library also provides a newtype-generating macro, which then adds another level of indirection. The patterns compete with each other; pick one.

6. **`serde::Convert` requires both `In: TryFrom<Out>` and `Out: TryFrom<In>`** on every impl block (`eventually-core/src/serde.rs:76-104`). The `Serializer` impl only needs `Out: TryFrom<In>`, and the `Deserializer` impl only `In: TryFrom<Out>`. Splitting the bounds would make the type usable in half-direction scenarios.

7. **Error types use `anyhow::Error` inside `Internal` variants** (`GetError::Internal`, `SaveError::Internal`, `AppendError::Internal`). This leaks `anyhow` into the public surface and forces downstream crates to dig through `source()` to recover the typed error. Consider either `Box<dyn Error + Send + Sync + 'static>` or a generic `Internal<E>` parameter. The `eventually-postgres/tests/aggregate_repository.rs:552` test already does the ugly `error.source().is_some_and(...)` dance.

8. **`async_trait` everywhere.** With MSRV high enough (Edition 2021, current Rust stable supports async fn in traits), this is worth revisiting. At minimum, document MSRV (it's missing from every `Cargo.toml`).

9. **`aggregate::test::Scenario` forces `R: From<Root<T>> + Deref<Target=Root<T>>`** (`aggregate/test.rs:156`). That's exactly the shape the `aggregate_root` macro *should* produce — but since the macro doesn't compile today, and users can use plain `Root<T>` (which trivially satisfies both), the test module effectively has no non-macro consumer.

10. **`InMemory` event store panics on poisoned locks** (`event/store.rs:135,167`). `.expect(...)` on an `RwLock` read/write is appropriate in single-threaded happy-path code, but a library that's explicitly optimised for "test doubles" should surface an error or use `parking_lot`.

11. **`eventually-postgres` `check_for_conflict_error`** parses the `RAISE EXCEPTION` message with a regex (`eventually-postgres/src/lib.rs:21`). That's extremely fragile — one translation or locale change and the mapping breaks silently. Use a PostgreSQL custom `SQLSTATE` (e.g. `P0001` with a structured message) or raise with `ERRCODE` you define.

12. **`aggregate::Repository` Postgres impl stores _only the latest snapshot_, and then also appends the events** (`eventually-postgres/src/aggregate.rs:192-204`). Yet the `get` method *only reads the snapshot* and does not replay events. That means:
    - The Postgres repository isn't actually event-sourced — snapshots are authoritative.
    - The `events` table ends up being write-only from the `get` path.
    - Snapshots are written on every single command, so the table has no throttling/point-in-time strategy.
    This may be intentional (hybrid snapshot model), but it's inconsistent with the `EventSourced` repository in `eventually-core`, and it's not documented.

13. **`Version` is returned as `i32` from Postgres** but the trait is `u64`. The migrations add `CHECK ("version" > 0)` which is correct but then `SELECT version` binds into `i32` and casts back up — using `BIGINT` (i64) would match the domain and remove a class of bugs.

14. **Transaction handling in Postgres `append`** explicitly `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE`. Fine for correctness but should be called out in docs; `DEFERRABLE` in particular is a subtle choice (it waits for serializable snapshot rather than failing eagerly).

15. **`command::Handler` is typed with a single command type `T`**. That's DDD-pure but forces a `Service` struct to `impl Handler<CreateUser>`, `impl Handler<ChangeUserPassword>`, etc. It scales fine but makes dispatching commands from a transport layer (gRPC/HTTP) verbose. Consider adding an `enum Command { ... }`-friendly helper or a `Dispatcher` trait.

## 3. Idiomatic Rust

- Lints are strict (`deny(clippy::pedantic, clippy::cargo)`) — a good choice — but then every backend has cast-permitting `#[allow(...)]`s. Either invest in a newtype `Version`/`StreamVersion`, or loosen the lint and document invariants.
- `pub use eventually_core::{aggregate, command, event, message, query, serde, version};` from `eventually/src/lib.rs:10` is a nice facade. But `eventually-core` is published separately, so users can pick the wrong crate to depend on. Consider renaming `eventually-core` to `eventually-internal` and marking it `publish = false` — or keeping it public but clearly documenting that `eventually` is the supported entry point. The macros' reliance on a specific crate path makes this more urgent.
- `futures::future::ready` + `.and_then` combinator in `eventually-postgres/src/event.rs:202` is fine but a simple `try_stream!` or explicit `async_stream` block would be more readable.
- `async-trait` is used consistently — but the `Handler<T>` blanket impl for `F: Fn(Envelope<T>) -> Fut` (`command/mod.rs:51`) requires `Fut: Send + Sync` — requiring `Sync` on a future is almost always wrong (futures are typically `!Sync`). Same in `query.rs:44`.
- `thiserror` usage is correct but overuses `#[source]` where `#[from]` would be idiomatic, and mixes `#[from]` on some variants (`#[from] version::ConflictError`) and not others.
- `env_logger`/`tracing` integration: the `InstrumentedAggregateRepository` requires `T: Debug`, `T::Id: Debug`, `T::Event: Debug`. For the tracing instrument's `ret, err`, this is necessary — but those bounds leak into users who didn't ask for tracing. Consider `#[instrument(skip_all)]` and manual `debug!` fields.
- Documentation examples are `// text` blocks (not `rust` blocks) everywhere. `cargo test --doc` will not exercise them, and `docs.rs` won't highlight syntax. Making them `rust,ignore` is already strictly better.
- `eventually-postgres` is missing a module-level doc comment and has the `#![warn(missing_docs)]` toned down from `deny` — the rest of the workspace `deny`s. This is a lint inconsistency.

## 4. Documentation & onboarding

- README says "v0.5.0 is under active development" and tells users to install from a git dependency. For a 4-year-old crate still shipping from git, that's a trust issue. Either cut v0.5 or document that 0.4.x is the latest stable on crates.io and link to its docs.
- No `CHANGELOG.md`. There's a `CONTRIBUTING.md` but no `DEVELOPMENT.md` or `docs/` describing the conceptual model (Aggregate vs Root, why you need the newtype, when to use `rehydrate_from_state` vs event-sourced rehydration).
- README references a docs-hosted URL at `https://get-eventually.github.io/eventually-rs/eventually` built by `.github/workflows/docs.yml` — but that uses `GITHUB_TOKEN` as `personal_token`, which only works for pushes the bot can perform. A purpose-built `actions/deploy-pages@v4` using the official Pages flow would be more reliable.
- Examples: only `bank-accounting`, and it depends on OpenTelemetry OTLP, gRPC, docker-compose — a lot of surface just to see the pattern. A minimal example (maybe a 100-line `todolist` — you have a branch for that!) would dramatically lower the barrier to entry.

## 5. Repository setup

### Good
- `rustfmt.toml` with `group_imports = "StdExternalCrate"` + `imports_granularity = "Module"` — very nice baseline.
- `.clippy.toml` with `allowed-duplicate-crates` is a pragmatic workaround for `clippy::cargo`.
- `renovate.json5` replaces Dependabot (you disabled it in `b57bdae`). Good configuration, grouping non‑major and GH Actions.
- `flake.nix` / `.envrc` for reproducible dev shells is a nice touch.
- `codecov.yml` excludes examples — correct.

### Missing or weak

1. **No release automation.** This is the headline issue you called out.
   - No `release.yml` workflow, no `cargo publish` job, no [release-plz](https://release-plz.ieni.dev/), no [release-please](https://github.com/googleapis/release-please-action), no tag-triggered workflow.
   - With 4 workspace crates (`eventually`, `eventually-core`, `eventually-macros`, `eventually-postgres`), you need ordered publish and synchronised version bumps. **Strong recommendation: adopt `release-plz`.** It understands Cargo workspaces, handles conventional commits (which your `CONTRIBUTING.md` already mandates), produces a "release PR", writes a changelog, and publishes crates in dependency order. It integrates cleanly with your existing renovate / conventional-commit setup.
   - Alternative: `cargo-smart-release` or `cargo-workspaces publish --from-git`.

2. **CI uses `actions-rs/*` actions (`ci.yml:31,41,76,112,120,138,149`) which are archived and unmaintained since 2022.** Migrate to `dtolnay/rust-toolchain@stable` and call `cargo` directly; or use `actions-rust-lang/setup-rust-toolchain`.

3. **`cargo test --workspace --all-features`** in CI does not actually have a Postgres service bound to the matrix test job (`ci.yml:8-47`). The `coverage` job has `services.postgres`, but the `test` job doesn't. If any `eventually-postgres` test shelled out, the matrix job would fail. The tests use `testcontainers` (good) so this is fine now, but CI must have Docker-in-Docker available — which `ubuntu-latest` does by default, and this should be documented.

4. **`.github/dependabot.yml` disabled**, but no MSRV job, no `cargo deny` / `cargo audit` (especially pertinent given the recent `sqlx` security update).

5. **No CODEOWNERS file**, no issue labels enforced, no PR template. The `CONTRIBUTING.md` mentions labels per crate but nothing provisions them.

6. **Branch hygiene.** 11 open branches (`feat/public-crate-api`, `fix/compile-issues`, `chore/renovate-*`, dependabot branches predating the dependabot disable, `test/entity-mod`, `v0.5.0/command-error-recorder`). Several look abandoned. A clean sweep and a `main`-only policy with branch protection would help.

7. **`main` currently has a `wip` commit** (`7a4cd2a`). If that's on the remote, it's a maintainability red flag. At minimum, use `git reset --hard origin/main` locally and do a clean v0.5 PR.

8. **No MSRV.** Declare `rust-version = "1.75"` (or whatever you actually need — `LazyLock` in `event/store.rs:346` requires 1.80+) in every `Cargo.toml`, and test it in CI via a matrix entry.

9. **`Cargo.toml`s are missing `homepage`, `documentation`, `rust-version`, `include`**. Before publishing 0.5 you'll want to ensure crates.io shows the right metadata.

10. **Dual-licensing is conventional in Rust ecosystem** (MIT + Apache-2.0). You use MIT-only, which is fine but unusual and slightly limits adoption in Apache-2.0-required orgs.

11. **`docs.yml` only runs on `main`** — gh-pages can drift silently if it breaks on a PR. Consider running `cargo doc --no-deps -D warnings` as part of `ci.yml`.

## 6. Prioritised recommendations

**P0 — make it build and publishable again**
1. Fix the `aggregate_root` macro: emit `::eventually_core::aggregate::Root<...>` (or `::eventually::...`) and decide on the "one true crate path". Reconcile the double-Deref field access issue in all examples and tests — either drop the newtype pattern, or provide an accessor method (`self.state()`, `self.state_mut()`) and rewrite examples to use it.
2. Drop the `wip` commit from `main`; create a clean 0.5 branch PR.
3. Add `release-plz` workflow + a `release.yml` on tag. Publish `eventually-core → eventually-macros → eventually-postgres → eventually` in order.

**P1 — API hygiene before freezing 0.5**
4. Introduce `Version` as a newtype; remove all cast-allow attributes.
5. Reconsider `Internal(anyhow::Error)` in public error enums — either expose it as `Box<dyn Error>` or parameterise.
6. Replace regex-based conflict detection in `eventually-postgres` with a dedicated `SQLSTATE` code.
7. Document the Postgres snapshot-vs-event-sourcing model, or make it purely event-sourced.
8. Split `serde::Convert` bounds per direction.
9. Drop `+ Sync` from future bounds in `Handler` blanket impls.

**P2 — developer/maintainer experience**
10. Migrate CI off archived `actions-rs/*`.
11. Add `cargo-deny` and `cargo-audit` workflows.
12. Add an MSRV matrix entry, declare `rust-version` in every `Cargo.toml`.
13. Add a minimal `examples/todolist` (you already have a branch).
14. Add `CHANGELOG.md` (release-plz generates this for you).
15. Consider dual-licensing MIT/Apache-2.0.
16. Prune stale branches; enable branch protection on `main`.
17. Turn doc comment examples from `// text` to `rust,ignore` or make them actually compile.

---

## Quick "stop-doing / start-doing" list

**Stop:**
- Publishing a crate that doesn't compile on HEAD.
- Relying on a regex over a locale-dependent PostgreSQL error message.
- Leaving `wip` commits on `main`.
- Using archived CI actions.

**Start:**
- Adopting `release-plz` and conventional-commit-driven releases.
- Using a `Version` newtype throughout.
- Documenting MSRV and adding it to CI.
- Writing one minimal example that compiles in under 50 LOC.

The core abstractions are genuinely good. The path from "abandoned 0.5-wip" to "shippable 0.5" is mostly packaging, build hygiene, and a handful of targeted API fixes — not a rewrite.
