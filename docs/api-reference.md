# API Reference — mcp-banking

## Accounts

### `list_accounts`
List all linked bank accounts. No parameters.

### `get_account`
| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `get_balances`
Get real-time balances for all linked accounts. No parameters.

---

## Transactions

### `list_transactions`
| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `account_id` | String | Yes | Account ID |
| `from` | String | Yes | Start date (YYYY-MM-DD) |
| `to` | String | Yes | End date (YYYY-MM-DD) |
| `limit` | u32 | No | Max results (default: 50) |

### `get_transaction`
| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `search_transactions`
| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `query` | String | Yes | Search by description/merchant |
| `limit` | u32 | No | Max results (default: 50) |

### `categorize_transaction`
| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

Returns the transaction with its category.

### `sync_transactions`
Trigger a refresh from the bank. No parameters. Returns sync summary.

---

## Identity

### `get_identity`
Get account holder identity (name, email, phone, address). No parameters.

---

## Institutions

### `list_institutions`
| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `country` | String | No | ISO country code (e.g. "US", "NG") |
| `limit` | u32 | No | Max results (default: 20) |

### `get_institution`
| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

---

## Payments

### `initiate_payment`
**Risk: financial_action, requires approval.**

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `account_id` | String | Yes | Source account |
| `amount` | f64 | Yes | Amount to transfer |
| `currency` | String | No | ISO currency (default: USD) |
| `recipient` | String | Yes | Recipient account/ID |
| `reference` | String | Yes | Payment reference |

### `get_payment_status`
| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `list_payments`
| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

---

## Statements

### `get_statement`
| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `account_id` | String | Yes | Account ID |
| `from` | String | Yes | Period start (YYYY-MM-DD) |
| `to` | String | Yes | Period end (YYYY-MM-DD) |

---

## Shared Types

### BankAccount
```json
{"id": "acc-1", "name": "Plaid Checking", "account_type": "depository", "subtype": "checking", "mask": "0000", "currency": "USD", "balance_available": 100.0, "balance_current": 110.0, "backend": "plaid"}
```

### Transaction
```json
{"id": "txn-1", "account_id": "acc-1", "amount": 5.40, "currency": "USD", "date": "2026-05-14", "description": "Uber", "category": "Travel", "merchant_name": "Uber", "pending": false, "backend": "plaid"}
```

### Payment
```json
{"id": "pay-1", "status": "created", "amount": 500.0, "currency": "USD", "recipient": "vendor-id", "reference": "Invoice 123", "backend": "plaid"}
```
