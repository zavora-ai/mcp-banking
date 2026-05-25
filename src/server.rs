//! MCP tool router for banking operations.
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::types::BankingBackend;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTransactionsInput { pub account_id: String, pub from: String, pub to: String, #[serde(default = "d50")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchInput { pub query: String, #[serde(default = "d50")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInstitutionsInput { #[serde(default)] pub country: Option<String>, #[serde(default = "d20")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InitiatePaymentInput { pub account_id: String, pub amount: f64, #[serde(default = "d_usd")] pub currency: String, pub recipient: String, pub reference: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPaymentsInput { #[serde(default = "d20")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StatementInput { pub account_id: String, pub from: String, pub to: String }

fn d20() -> u32 { 20 }
fn d50() -> u32 { 50 }
fn d_usd() -> String { "USD".into() }

#[derive(Clone)]
pub struct BankingServer { pub backend: Arc<dyn BankingBackend> }

#[tool_router(server_handler)]
impl BankingServer {
    #[tool(description = "List linked bank accounts")]
    async fn list_accounts(&self) -> String {
        match self.backend.list_accounts().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get a bank account by ID with balance")]
    async fn get_account(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_account(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get real-time balances for all linked accounts")]
    async fn get_balances(&self) -> String {
        match self.backend.get_balances().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "List transactions for an account with date range")]
    async fn list_transactions(&self, Parameters(i): Parameters<ListTransactionsInput>) -> String {
        match self.backend.list_transactions(&i.account_id, &i.from, &i.to, i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get a single transaction by ID")]
    async fn get_transaction(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_transaction(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Search transactions by description, amount, or category")]
    async fn search_transactions(&self, Parameters(i): Parameters<SearchInput>) -> String {
        match self.backend.search_transactions(&i.query, i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get or update the category of a transaction")]
    async fn categorize_transaction(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.categorize_transaction(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get account holder identity (name, address, email)")]
    async fn get_identity(&self) -> String {
        match self.backend.get_identity().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "List supported financial institutions")]
    async fn list_institutions(&self, Parameters(i): Parameters<ListInstitutionsInput>) -> String {
        match self.backend.list_institutions(i.country.as_deref(), i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get details about a financial institution")]
    async fn get_institution(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_institution(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Initiate a bank transfer/payment (draft, requires approval)")]
    async fn initiate_payment(&self, Parameters(i): Parameters<InitiatePaymentInput>) -> String {
        match self.backend.initiate_payment(&i.account_id, i.amount, &i.currency, &i.recipient, &i.reference).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get the status of a payment/transfer")]
    async fn get_payment_status(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_payment_status(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "List payment/transfer history")]
    async fn list_payments(&self, Parameters(i): Parameters<ListPaymentsInput>) -> String {
        match self.backend.list_payments(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Get a bank statement for a period")]
    async fn get_statement(&self, Parameters(i): Parameters<StatementInput>) -> String {
        match self.backend.get_statement(&i.account_id, &i.from, &i.to).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }
    #[tool(description = "Trigger a transaction sync/refresh from the bank")]
    async fn sync_transactions(&self) -> String {
        match self.backend.sync_transactions().await { Ok(v) => v, Err(e) => format!("Error: {e}") }
    }
}

#[async_trait::async_trait]
impl HealthCheck for BankingServer {
    async fn check_health(&self) -> HealthStatus {
        match self.backend.list_accounts().await {
            Ok(_) => HealthStatus { healthy: true, message: Some(format!("{} connected", self.backend.name())), latency_ms: Some(1) },
            Err(e) => HealthStatus { healthy: false, message: Some(format!("{}: {e}", self.backend.name())), latency_ms: None },
        }
    }
}
