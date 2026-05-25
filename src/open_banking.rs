//! Open Banking UK/EU backend (PSD2 compliant).
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct OpenBankingBackend { http: Client, base_url: String, token: String }

impl OpenBankingBackend {
    pub fn new(base_url: String, token: String) -> Self {
        Self { http: Client::new(), base_url: base_url.trim_end_matches('/').to_string(), token }
    }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{}/{path}", self.base_url)).bearer_auth(&self.token).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{}/{path}", self.base_url)).bearer_auth(&self.token).json(body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl BankingBackend for OpenBankingBackend {
    fn name(&self) -> &str { "open_banking" }

    async fn list_accounts(&self) -> Result<Vec<BankAccount>> {
        let resp = self.get("accounts").await?;
        Ok(resp["Data"]["Account"].as_array().map(|a| a.iter().map(|acc| BankAccount { id: acc["AccountId"].as_str().unwrap_or("").into(), name: acc["Nickname"].as_str().or(acc["Description"].as_str()).unwrap_or("").into(), account_type: acc["AccountType"].as_str().map(Into::into), subtype: acc["AccountSubType"].as_str().map(Into::into), mask: acc["Account"].as_array().and_then(|a| a.first()).and_then(|a| a["Identification"].as_str()).map(|s| s[s.len().saturating_sub(4)..].to_string()), currency: acc["Currency"].as_str().map(Into::into), balance_available: None, balance_current: None, institution_id: None, institution_name: None, backend: "open_banking".into() }).collect()).unwrap_or_default())
    }

    async fn get_account(&self, id: &str) -> Result<BankAccount> {
        let resp = self.get(&format!("accounts/{id}")).await?;
        let acc = &resp["Data"]["Account"].as_array().and_then(|a| a.first()).ok_or_else(|| anyhow::anyhow!("not found"))?;
        Ok(BankAccount { id: acc["AccountId"].as_str().unwrap_or("").into(), name: acc["Nickname"].as_str().unwrap_or("").into(), account_type: acc["AccountType"].as_str().map(Into::into), subtype: acc["AccountSubType"].as_str().map(Into::into), mask: None, currency: acc["Currency"].as_str().map(Into::into), balance_available: None, balance_current: None, institution_id: None, institution_name: None, backend: "open_banking".into() })
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let resp = self.get("balances").await?;
        Ok(resp["Data"]["Balance"].as_array().map(|a| a.iter().map(|b| Balance { account_id: b["AccountId"].as_str().unwrap_or("").into(), account_name: None, available: b["Amount"]["Amount"].as_str().and_then(|s| s.parse().ok()).filter(|_| b["CreditDebitIndicator"].as_str() == Some("Credit")), current: b["Amount"]["Amount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: b["Amount"]["Currency"].as_str().map(Into::into), backend: "open_banking".into() }).collect()).unwrap_or_default())
    }

    async fn list_transactions(&self, account_id: &str, from: &str, to: &str, _limit: u32) -> Result<Vec<Transaction>> {
        let resp = self.get(&format!("accounts/{account_id}/transactions?fromBookingDateTime={from}T00:00:00Z&toBookingDateTime={to}T23:59:59Z")).await?;
        Ok(resp["Data"]["Transaction"].as_array().map(|a| a.iter().map(|t| Transaction { id: t["TransactionId"].as_str().unwrap_or("").into(), account_id: t["AccountId"].as_str().unwrap_or("").into(), amount: t["Amount"]["Amount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0) * if t["CreditDebitIndicator"].as_str() == Some("Debit") { -1.0 } else { 1.0 }, currency: t["Amount"]["Currency"].as_str().map(Into::into), date: t["BookingDateTime"].as_str().unwrap_or("").into(), description: t["TransactionInformation"].as_str().unwrap_or("").into(), category: t["BankTransactionCode"]["Code"].as_str().map(Into::into), merchant_name: t["MerchantDetails"]["MerchantName"].as_str().map(Into::into), pending: t["Status"].as_str() == Some("Pending"), transaction_type: t["CreditDebitIndicator"].as_str().map(Into::into), backend: "open_banking".into() }).collect()).unwrap_or_default())
    }

    async fn get_transaction(&self, id: &str) -> Result<Transaction> {
        let accounts = self.list_accounts().await?;
        for acc in &accounts {
            let txns = self.list_transactions(&acc.id, "2020-01-01", "2030-12-31", 500).await?;
            if let Some(t) = txns.into_iter().find(|t| t.id == id) { return Ok(t); }
        }
        anyhow::bail!("transaction not found")
    }

    async fn search_transactions(&self, query: &str, _limit: u32) -> Result<Vec<Transaction>> {
        let accounts = self.list_accounts().await?;
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for acc in &accounts {
            let txns = self.list_transactions(&acc.id, "2020-01-01", "2030-12-31", 200).await?;
            results.extend(txns.into_iter().filter(|t| t.description.to_lowercase().contains(&q)));
        }
        Ok(results)
    }

    async fn categorize_transaction(&self, id: &str) -> Result<Transaction> { self.get_transaction(id).await }

    async fn get_identity(&self) -> Result<Identity> {
        let resp = self.get("party").await?;
        let p = &resp["Data"]["Party"];
        Ok(Identity { name: p["Name"].as_str().map(Into::into), email: p["Email"].as_str().map(Into::into), phone: p["Phone"].as_str().map(Into::into), address: None, backend: "open_banking".into() })
    }

    async fn list_institutions(&self, _country: Option<&str>, _limit: u32) -> Result<Vec<Institution>> {
        Ok(vec![]) // Open Banking doesn't have a standard institution directory
    }

    async fn get_institution(&self, _id: &str) -> Result<Institution> {
        anyhow::bail!("Open Banking does not provide an institution directory API")
    }

    async fn initiate_payment(&self, account_id: &str, amount: f64, currency: &str, recipient: &str, reference: &str) -> Result<Payment> {
        let resp = self.post("domestic-payments", &serde_json::json!({"Data": {"Initiation": {"InstructionIdentification": reference, "EndToEndIdentification": reference, "InstructedAmount": {"Amount": amount.to_string(), "Currency": currency}, "DebtorAccount": {"Identification": account_id}, "CreditorAccount": {"Identification": recipient}}}})).await?;
        Ok(Payment { id: resp["Data"]["DomesticPaymentId"].as_str().unwrap_or("").into(), status: resp["Data"]["Status"].as_str().unwrap_or("Pending").into(), amount, currency: Some(currency.into()), recipient: Some(recipient.into()), reference: Some(reference.into()), created_at: resp["Data"]["CreationDateTime"].as_str().map(Into::into), backend: "open_banking".into() })
    }

    async fn get_payment_status(&self, id: &str) -> Result<Payment> {
        let resp = self.get(&format!("domestic-payments/{id}")).await?;
        let d = &resp["Data"];
        Ok(Payment { id: d["DomesticPaymentId"].as_str().unwrap_or("").into(), status: d["Status"].as_str().unwrap_or("").into(), amount: d["Initiation"]["InstructedAmount"]["Amount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: d["Initiation"]["InstructedAmount"]["Currency"].as_str().map(Into::into), recipient: d["Initiation"]["CreditorAccount"]["Identification"].as_str().map(Into::into), reference: d["Initiation"]["InstructionIdentification"].as_str().map(Into::into), created_at: d["CreationDateTime"].as_str().map(Into::into), backend: "open_banking".into() })
    }

    async fn list_payments(&self, _limit: u32) -> Result<Vec<Payment>> {
        Ok(vec![]) // Open Banking doesn't have a list payments endpoint
    }

    async fn get_statement(&self, account_id: &str, from: &str, to: &str) -> Result<Statement> {
        let resp = self.get(&format!("accounts/{account_id}/statements?fromStatementDateTime={from}T00:00:00Z&toStatementDateTime={to}T23:59:59Z")).await?;
        let stmt = resp["Data"]["Statement"].as_array().and_then(|a| a.first());
        Ok(Statement { account_id: account_id.into(), period_start: from.into(), period_end: to.into(), opening_balance: stmt.and_then(|s| s["StartBalance"]["Amount"]["Amount"].as_str()).and_then(|s| s.parse().ok()), closing_balance: stmt.and_then(|s| s["EndBalance"]["Amount"]["Amount"].as_str()).and_then(|s| s.parse().ok()), transaction_count: 0, backend: "open_banking".into() })
    }

    async fn sync_transactions(&self) -> Result<String> {
        Ok("Open Banking transactions are real-time — no sync needed".into())
    }
}
