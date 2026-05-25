//! Unified banking types shared across all backends.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: String,
    pub name: String,
    pub account_type: Option<String>,
    pub subtype: Option<String>,
    pub mask: Option<String>,
    pub currency: Option<String>,
    pub balance_available: Option<f64>,
    pub balance_current: Option<f64>,
    pub institution_id: Option<String>,
    pub institution_name: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub date: String,
    pub description: String,
    pub category: Option<String>,
    pub merchant_name: Option<String>,
    pub pending: bool,
    pub transaction_type: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub account_id: String,
    pub account_name: Option<String>,
    pub available: Option<f64>,
    pub current: f64,
    pub currency: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub country: Option<String>,
    pub url: Option<String>,
    pub logo: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub status: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub recipient: Option<String>,
    pub reference: Option<String>,
    pub created_at: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub account_id: String,
    pub period_start: String,
    pub period_end: String,
    pub opening_balance: Option<f64>,
    pub closing_balance: Option<f64>,
    pub transaction_count: u32,
    pub backend: String,
}

/// Backend trait — each banking provider implements this.
#[async_trait::async_trait]
pub trait BankingBackend: Send + Sync {
    fn name(&self) -> &str;

    // Accounts
    async fn list_accounts(&self) -> anyhow::Result<Vec<BankAccount>>;
    async fn get_account(&self, id: &str) -> anyhow::Result<BankAccount>;
    async fn get_balances(&self) -> anyhow::Result<Vec<Balance>>;

    // Transactions
    async fn list_transactions(&self, account_id: &str, from: &str, to: &str, limit: u32) -> anyhow::Result<Vec<Transaction>>;
    async fn get_transaction(&self, id: &str) -> anyhow::Result<Transaction>;
    async fn search_transactions(&self, query: &str, limit: u32) -> anyhow::Result<Vec<Transaction>>;
    async fn categorize_transaction(&self, id: &str) -> anyhow::Result<Transaction>;

    // Identity
    async fn get_identity(&self) -> anyhow::Result<Identity>;

    // Institutions
    async fn list_institutions(&self, country: Option<&str>, limit: u32) -> anyhow::Result<Vec<Institution>>;
    async fn get_institution(&self, id: &str) -> anyhow::Result<Institution>;

    // Payments
    async fn initiate_payment(&self, account_id: &str, amount: f64, currency: &str, recipient: &str, reference: &str) -> anyhow::Result<Payment>;
    async fn get_payment_status(&self, id: &str) -> anyhow::Result<Payment>;
    async fn list_payments(&self, limit: u32) -> anyhow::Result<Vec<Payment>>;

    // Statements
    async fn get_statement(&self, account_id: &str, from: &str, to: &str) -> anyhow::Result<Statement>;

    // Sync
    async fn sync_transactions(&self) -> anyhow::Result<String>;
}
