# Changelog

## 0.1.0

First release: what every Zerpto service shares, on Loco 1.3 with Postgres.

**Always on**

- `http`: `Problem` / `ApiResult`, the `{data, meta}` envelope, page and cursor
  paging, id encoding (`ApiIds`), idempotency (`IdempotencyKey`), filters
  (`QueryParam`), `Nullable`, `CurrentSession` and the `Authorizer` trait it
  asks. Each service defines its own ID prefixes.
- Error contract: every `/v1` error is `{error: {message, request_id}}`, plus
  `details` (`{field, message}`) on a failed validation (`422`). There is no
  `error.code` or `error.field`; clients go by the HTTP status.
- `validation`: `ValidatedJson<T>`, `ValidationDetails`, and the shared rules
  `name`, `code`, `amount`, `tax_id`, `date` and `date_range`.
- `helpers`: money formatting and parsing, text and file name cleaning, the
  success and file download responses, and a lazy gRPC channel.
- `grpc`: the health service and `GrpcSettings::bind_addr`.
- `settings::CoreSettings`, with `CoreSettings::section` for feature blocks, and
  `bootstrap::after_context`.
- Initializers:
  - `Sentry` and `QueryLog`.
  - `GrpcServer`: serves the services a service passes, beside the health
    service.
  - `Authorization`: registers the authorizer. Without one,
    `CurrentSession::require_any` answers `503`.
  - `NotFound`: unmatched routes answer with the JSON `404` envelope. It needs
    `server.middlewares.fallback.enable: false`.

**Optional features**

- `events`: transactional outbox (entity, model, migration), relay and consumer
  on NATS JetStream, the `Events` initializer and the `OutboxCleanup` task.
- `storage`: local disk and Cloudflare R2 backends, file type sniffing, files,
  multipart uploads and signed links.
- `iam`: `IamAuthorizer`, the gRPC client of the IAM service, configured by
  `settings.iam.url`. The contract comes from `zerpto-proto` v0.1.0.
- `relay`: `RelayMessage` queues a job to the relay service, configured by
  `settings.relay`.
