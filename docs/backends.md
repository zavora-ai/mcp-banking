# Backends — mcp-banking

## Backend Selection Priority

1. Plaid (`PLAID_CLIENT_ID` + `PLAID_SECRET` + `PLAID_ACCESS_TOKEN`)
2. Mono (`MONO_SECRET_KEY` + `MONO_ACCOUNT_ID`)
3. Open Banking (`OB_BASE_URL` + `OB_TOKEN`)

---

## Plaid

**Region:** US, Canada, UK, EU  
**Feature flag:** `plaid` (default)  
**API:** REST, JSON  
**Auth:** Client ID + Secret per request

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `PLAID_CLIENT_ID` | Yes | From Plaid Dashboard |
| `PLAID_SECRET` | Yes | Environment-specific secret |
| `PLAID_ACCESS_TOKEN` | Yes | Obtained via Link token exchange |
| `PLAID_ENV` | No | `sandbox` (default), `development`, `production` |

### Getting Credentials

1. Sign up at [dashboard.plaid.com](https://dashboard.plaid.com)
2. Get Client ID and Secret from Keys page
3. Use Plaid Link to connect a bank and get an access token
4. For testing: use sandbox with `user_good` / `pass_good`

### Sandbox Testing

```bash
# Create sandbox access token (no browser needed)
curl -X POST https://sandbox.plaid.com/sandbox/public_token/create \
  -H "Content-Type: application/json" \
  -d '{"client_id":"YOUR_ID","secret":"YOUR_SECRET","institution_id":"ins_109508","initial_products":["transactions","auth"]}'

# Exchange for access token
curl -X POST https://sandbox.plaid.com/item/public_token/exchange \
  -H "Content-Type: application/json" \
  -d '{"client_id":"YOUR_ID","secret":"YOUR_SECRET","public_token":"PUBLIC_TOKEN_FROM_ABOVE"}'
```

### API Mapping

| Tool | Plaid Endpoint |
|------|---------------|
| list_accounts | `POST /accounts/get` |
| get_balances | `POST /accounts/balance/get` |
| list_transactions | `POST /transactions/get` |
| sync_transactions | `POST /transactions/sync` |
| get_identity | `POST /identity/get` |
| list_institutions | `POST /institutions/get` |
| initiate_payment | `POST /payment_initiation/payment/create` |

---

## Mono

**Region:** Nigeria, Kenya, Ghana, South Africa  
**Feature flag:** `mono` (default)  
**API:** REST v2, JSON  
**Auth:** Secret key in `mono-sec-key` header

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `MONO_SECRET_KEY` | Yes | From Mono Dashboard (test_sk_* or live_sk_*) |
| `MONO_ACCOUNT_ID` | Yes | Account ID from Connect widget callback |

### Getting Credentials

1. Sign up at [app.withmono.com](https://app.withmono.com)
2. Get Secret Key from Settings → API Keys
3. Use Mono Connect widget to link a bank account
4. Account ID is returned via webhook after successful linking

### Connecting a Bank Account

```bash
# 1. Initiate connection
curl -X POST https://api.withmono.com/v2/accounts/initiate \
  -H "mono-sec-key: YOUR_SECRET" \
  -H "Content-Type: application/json" \
  -d '{"customer":{"name":"Name","email":"email@example.com"},"scope":"auth","redirect_url":"http://localhost:8856/callback"}'

# 2. Open the mono_url in browser, complete bank linking
# 3. Account ID arrives via webhook or customer accounts endpoint
```

### Notes

- Amounts are in **kobo/cents** (divide by 100 for display currency)
- Supported banks: KCB, GTBank, Access Bank, First Bank, UBA, Zenith, and 100+ more
- Test mode uses `test_sk_*` keys with simulated bank data

### API Mapping

| Tool | Mono Endpoint |
|------|--------------|
| get_account | `GET /v2/accounts/{id}` |
| get_balances | `GET /v2/accounts/{id}/balance` |
| list_transactions | `GET /v2/accounts/{id}/transactions` |
| get_identity | `GET /v2/accounts/{id}/identity` |
| get_statement | `POST /v2/accounts/{id}/statement` |
| initiate_payment | `POST /v2/payments/initiate` |
| sync_transactions | `POST /v2/accounts/{id}/sync` |

---

## Open Banking (UK/EU)

**Region:** UK, EU (PSD2 compliant)  
**Feature flag:** `open-banking`  
**API:** REST, OBP Standard  
**Auth:** OAuth2 Bearer token

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `OB_BASE_URL` | Yes | Bank's Open Banking API base URL |
| `OB_TOKEN` | Yes | OAuth2 access token |

### Getting Credentials

1. Register as a TPP (Third Party Provider) with your national authority
2. Register with the bank's developer portal
3. Complete OAuth2 consent flow with the account holder
4. Use the access token for API calls

### Supported Banks (examples)

| Bank | Base URL |
|------|----------|
| HSBC | `https://api.ob.hsbc.co.uk/obie/open-banking/v3.1` |
| Barclays | `https://openbanking.barclays.com/open-banking/v3.1` |
| Lloyds | `https://secure-api.lloydsbank.com/open-banking/v3.1` |
| NatWest | `https://ob.natwest.com/open-banking/v3.1` |
| Revolut | `https://openbanking.revolut.com/open-banking/v3.1` |

### Notes

- Follows UK Open Banking Implementation Entity (OBIE) standard
- Transactions use `CreditDebitIndicator` (amounts are always positive, direction indicated separately)
- Payment initiation requires separate consent
- No institution directory API — use bank's developer portal

### API Mapping

| Tool | Open Banking Endpoint |
|------|----------------------|
| list_accounts | `GET /accounts` |
| get_balances | `GET /balances` |
| list_transactions | `GET /accounts/{id}/transactions` |
| get_identity | `GET /party` |
| get_statement | `GET /accounts/{id}/statements` |
| initiate_payment | `POST /domestic-payments` |
| get_payment_status | `GET /domestic-payments/{id}` |
