# Agent guide: zerpto-core

## What this is

`zerpto-core` is the **shared crate of the Zerpto microservices**: what every
service needs, so a service holds only its own domain code. Services depend on
it by **git tag** and update by bumping the tag. Rust, on top of
[Loco](https://loco.rs). The template that uses it is
`/Users/fathur/Codes/com.zerpto.rstpl`; the monolith being split is
`/Users/fathur/Codes/com.zerpto.api-rs` (read-only).

## Hard rules

1. **Do not run any `cargo` command in this project** (build, check, clippy,
   fmt, test, nextest, run, in any scope or in the background). Write the code
   and tell the user which commands to run.
2. **Never run tests.** Write and update them; the user runs them.
3. **Ask when unsure** about a shared contract. Do not guess.
4. **Never copy secrets** from the monolith.

## What belongs here

Only things that are identical in every service and carry no domain knowledge:

- `http`: `Problem` / `ApiResult`, the `{data, meta}` envelope, paging, id
  encoding (`ApiIds`, no prefixes), idempotency, filters, `Nullable`,
  `CurrentSession` and the `Authorizer` trait it asks.
- `helpers`: standalone functions with no shared state (money formatting, the
  success response, file naming, ...).
- `validation`: `ValidatedJson`, the shared rules and the validator-to-`Problem`
  mapping.
- `grpc`: the health service (proto owned here) and `GrpcSettings::bind_addr`.
  The `GrpcServer` initializer serves it beside the services the service passes.
- `iam` (feature `iam`): client of the IAM service
  (`IamAuthorizer`, contract from `zerpto-proto`, `settings.iam`).
- `events` (feature `events`): outbox (entity, model, migration), relay,
  consumer, `OutboxCleanup` task.
- `storage` (feature `storage`): backend, sniffing, files, uploads, signed links.
- `relay` (feature `relay`): job to the relay service.
- `settings::CoreSettings`, `bootstrap::after_context`, and the `Sentry`,
  `QueryLog`, `Events`, `GrpcServer`, `Authorization`, `NotFound` initializers.

**Loose coupling.** The core names no other service. `CoreSettings` holds only
generic blocks; anything about a peer (its address, queue, contract) lives in
that peer's feature module with its own settings block, read with
`CoreSettings::section`. ID prefixes belong to the owning service.

Not here: any service's own models, controllers, protos or routes. If only one
service uses it, it stays in that service.

## Rules for changes

- **Additive only.** Every service upgrades at its own pace, so a release must
  not break a service on the previous tag within the same minor version. Add
  fields, functions and features; do not rename, remove or change behavior. A
  breaking change needs a new minor (pre-1.0) or major version and a note in
  `CHANGELOG.md` saying what services must change.
- **Protos:** never renumber or reuse a field; add new ones. A breaking change
  is a new `v2` package.
- **ID prefixes** are not defined here. A prefix is registered once and never
  reused for another kind of record across all services; the list is kept in
  the `com.zerpto.openapi` repo.
- **Outbox migration:** `m20261121_000001_outbox_events.rs` keeps its name, so
  databases that already ran it do not run it twice. Never edit it after a
  release; add a new migration file instead and export it from
  `events/outbox/mod.rs`.
- **Loco is pinned here.** Services must use the same `loco-rs` minor. Upgrade
  Loco here first, release, then bump services. SQLite is never used: keep
  `db-postgres` in the feature list and `db-sqlite` out.
- Models follow the service rule: single `id` UUIDv7 primary key, generated in
  `before_save` (see `events/outbox/model.rs`).
- **Structs for shared properties, helpers for the rest.** Functions that take
  the same parameters (a `ctx`, a `prefix`, a key, settings) are methods of a
  struct that holds them (`ApiIds`, `IdempotencyKey`, `Files`, `FileLinks`,
  `IamAuthorizer`, `OutboxRelay`, `QueryParam`, ...). A function that needs no
  shared state is a standalone helper: the root
  `helpers/`. Helpers never depend on a Cargo feature.
- **`mod.rs` only declares and re-exports.** Every struct, enum, trait and test gets
  its own file. The exception is `helpers/`: related standalone functions and
  their consts share one file per topic (`money`, `text`, `response`, `channel`).
- Keep `cfg(feature = "...")` gates on whole modules in `lib.rs`, not scattered
  through code.

## Error contract

Every `/v1` error is `{error: {message, request_id}}`, never inside `data`.
There is no `error.code` and no `error.field`: clients go by the HTTP status.
A failed validation (`422`) adds `details`.

`Problem` constructors still take a `code` (and `.field(..)`), so services keep
compiling, but both are only logged, never sent. Put per-field errors in
`details` (via `ValidatedJson` or `Problem::validation`).

Changing this shape is a breaking change (see "Rules for changes").

## Validation

Services put `validator` rules on request structs and take
`ValidatedJson<T>` in the handler. A failed check is always `422`:

```json
{"error": {"message": "The given data was invalid.",
  "request_id": "...", "details": [
    {"field": "code", "message": "Code must be 1 to 32 characters long."},
    {"field": "name", "message": "Name is required."}]}}
```

- `message` is constant; the first rule's text is never promoted. Clients
  recognise it by status `422` and the `details` array.
- `details` is sorted by field name; entries with no field (cross-field rules)
  come last with `"field": null`. Nested fields are `a.b`, list items `a[0].b`.
- `details` exists only on this error.
- Bad JSON syntax or a wrong content type is `400`; a missing field or wrong
  type is `422` with one fieldless detail.
- Shared rules live in `validation/rules/` (one file each) and must set an explicit
  stable `code` and message on the `ValidationError` (logged and available to
  services; not sent).

## Releasing

1. Update `CHANGELOG.md` and `version` in `Cargo.toml`.
2. Tag: `git tag vX.Y.Z && git push --tags`.
3. In each service: bump `tag = "vX.Y.Z"` in `[workspace.dependencies]`, then
   run its checks.

## Before you call a change done

Do not run these yourself (rule 1). Tell the user to run:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
