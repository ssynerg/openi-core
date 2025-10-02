# RFC-0002: Envelope & Transport

**Status:** Draft  
**Goal:** Canonical message format and signing.

## Envelope
```json
{
  "v": 1,
  "id": "ULID",
  "src": "agent://tenant/node/agent",
  "dest": "topic://...|agent://...",
  "ts": "RFC3339",
  "ctype": "mime or logical type",
  "headers": { "trace_id": "...", "ttl_ms": 5000, "scopes": "phi:read" },
  "payload": {},
  "sig": "base64"
}
