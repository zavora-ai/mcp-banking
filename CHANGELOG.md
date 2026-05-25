# Changelog

## [1.0.0] - 2026-05-25

### Added
- Initial release with 15 tools across 6 categories
- **3 backends:** Plaid (US/EU), Mono (Africa), Open Banking (UK/EU PSD2)
- Accounts: list, get, get_balances
- Transactions: list, get, search, categorize, sync
- Identity: get account holder info
- Institutions: list, get supported banks
- Payments: initiate (financial_action), get_status, list
- Statements: get for a period
- Feature flags — compile only the backends you need (default: plaid + mono)
- Manifest validation on startup (adk-mcp-sdk 0.1.3)
- Health check verifies backend connectivity
