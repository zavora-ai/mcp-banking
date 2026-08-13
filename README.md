# Banking MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-banking.svg)](https://crates.io/crates/mcp-banking)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Unified banking MCP server with **15 tools** across **3 backends** — Plaid (US/Canada/EU), Mono (Africa), and Open Banking (UK/EU PSD2). Accounts, transactions, balances, payments, identity, and statements.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-banking/main/docs/assets/architecture.svg" alt="MCP Banking Architecture" width="800"/>
</p>

## Key Principles

- **Unified schema** — agents see consistent types regardless of backend
- **Multi-region** — Plaid (US/Canada/EU), Mono (NG/KE/GH/ZA), Open Banking (UK/EU)
- **Financial governance** — payment initiation requires approval gates
- **No credential exposure** — tokens stay in env vars
- **Single binary** — no Node.js, no Python

## Tools (15)

| Category | Tools | Risk |
|----------|-------|------|
| Accounts | `list_accounts`, `get_account`, `get_balances` | read_only |
| Transactions | `list_transactions`, `get_transaction`, `search_transactions`, `categorize_transaction`, `sync_transactions` | read_only |
| Identity | `get_identity` | read_only |
| Institutions | `list_institutions`, `get_institution` | read_only |
| Payments | `initiate_payment`, `get_payment_status`, `list_payments` | financial_action / read_only |
| Statements | `get_statement` | read_only |

## Backends

| Backend | Region | Protocol | Auth | Default |
|---------|--------|----------|------|:---:|
| **Plaid** | US, Canada, EU | REST | Client ID + Secret | ✅ |
| **Mono** | Nigeria, Kenya, Ghana, SA | REST | Secret Key | ✅ |
| **Open Banking** | UK, EU (PSD2) | REST (OBP) | OAuth2 Bearer | ❌ |

## Installation

```bash
cargo install mcp-banking --features all-backends
```

### Feature flags

```bash
# Default: Plaid + Mono
cargo install mcp-banking

# All backends
cargo install mcp-banking --features all-backends

# Specific
cargo install mcp-banking --no-default-features --features plaid
```

## Configuration

### Plaid

```bash
export PLAID_CLIENT_ID="your-client-id"
export PLAID_SECRET="your-secret"
export PLAID_ACCESS_TOKEN="access-sandbox-xxx"
export PLAID_ENV="sandbox"  # sandbox | development | production
```

### Mono (Africa)

```bash
export MONO_SECRET_KEY="live_sk_xxx"
export MONO_ACCOUNT_ID="account-id-from-connect"
```

### Open Banking (UK/EU)

```bash
export OB_BASE_URL="https://api.yourbank.com/open-banking/v3.1"
export OB_TOKEN="your-oauth-token"
```

## Client Configuration

### Claude Desktop / Kiro / Cursor

```json
{
  "mcpServers": {
    "banking": {
      "command": "mcp-banking",
      "args": [],
      "env": {
        "PLAID_CLIENT_ID": "xxx",
        "PLAID_SECRET": "xxx",
        "PLAID_ACCESS_TOKEN": "access-sandbox-xxx",
        "PLAID_ENV": "sandbox"
      }
    }
  }
}
```

## Usage Examples

```
"Show me all my bank accounts"
→ list_accounts()

"What's my current balance?"
→ get_balances()

"Show transactions from last month"
→ list_transactions(account_id: "acc-1", from: "2026-04-01", to: "2026-04-30")

"Search for Uber transactions"
→ search_transactions(query: "uber")

"Initiate a payment of $500 to vendor"
→ initiate_payment(account_id: "acc-1", amount: 500, currency: "USD", recipient: "vendor-id", reference: "Invoice 123")

"What's the status of my payment?"
→ get_payment_status(id: "payment-id")

"Get my bank statement for Q1"
→ get_statement(account_id: "acc-1", from: "2026-01-01", to: "2026-03-31")
```

## Documentation

| Document | Description |
|----------|-------------|
| [API Reference](docs/api-reference.md) | All 15 tools with parameters and types |
| [Backends](docs/backends.md) | Setup guides for Plaid, Mono, Open Banking |
| [Architecture](docs/assets/architecture.svg) | System diagram |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |

## Registry Compliance

- **HealthCheck** — verifies backend connectivity
- **mcp-server.toml** — 15 tools with risk classes and credential bindings
- **Manifest validation** — startup fails fast on invalid manifest
- **Structured tracing** — `RUST_LOG` env-filter

## Contributors

| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P2 high-impact)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** `sync_transactions`
- **MRTR approvals:** `initiate_payment`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
