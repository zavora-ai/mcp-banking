//! Plaid API backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct PlaidBackend { http: Client, base_url: String, client_id: String, secret: String, access_token: String }

impl PlaidBackend {
    pub fn new(client_id: String, secret: String, access_token: String, environment: &str) -> Self {
        let base_url = match environment {
            "production" => "https://production.plaid.com",
            "development" => "https://development.plaid.com",
            _ => "https://sandbox.plaid.com",
        }.to_string();
        Self { http: Client::new(), base_url, client_id, secret, access_token }
    }

    async fn post(&self, path: &str, extra: serde_json::Value) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({"client_id": self.client_id, "secret": self.secret, "access_token": self.access_token});
        if let Some(obj) = extra.as_object() { for (k, v) in obj { body[k] = v.clone(); } }
        Ok(self.http.post(format!("{}/{path}", self.base_url)).json(&body).send().await?.error_for_status()?.json().await?)
    }

    async fn post_no_token(&self, path: &str, extra: serde_json::Value) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({"client_id": self.client_id, "secret": self.secret});
        if let Some(obj) = extra.as_object() { for (k, v) in obj { body[k] = v.clone(); } }
        Ok(self.http.post(format!("{}/{path}", self.base_url)).json(&body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl BankingBackend for PlaidBackend {
    fn name(&self) -> &str { "plaid" }

    async fn list_accounts(&self) -> Result<Vec<BankAccount>> {
        let resp = self.post("accounts/get", serde_json::json!({})).await?;
        Ok(resp["accounts"].as_array().map(|a| a.iter().map(|acc| BankAccount {
            id: acc["account_id"].as_str().unwrap_or("").into(), name: acc["name"].as_str().unwrap_or("").into(),
            account_type: acc["type"].as_str().map(Into::into), subtype: acc["subtype"].as_str().map(Into::into),
            mask: acc["mask"].as_str().map(Into::into), currency: acc["balances"]["iso_currency_code"].as_str().map(Into::into),
            balance_available: acc["balances"]["available"].as_f64(), balance_current: acc["balances"]["current"].as_f64(),
            institution_id: None, institution_name: None, backend: "plaid".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_account(&self, id: &str) -> Result<BankAccount> {
        let resp = self.post("accounts/get", serde_json::json!({"options": {"account_ids": [id]}})).await?;
        let acc = resp["accounts"].as_array().and_then(|a| a.first()).ok_or_else(|| anyhow::anyhow!("account not found"))?;
        Ok(BankAccount { id: acc["account_id"].as_str().unwrap_or("").into(), name: acc["name"].as_str().unwrap_or("").into(), account_type: acc["type"].as_str().map(Into::into), subtype: acc["subtype"].as_str().map(Into::into), mask: acc["mask"].as_str().map(Into::into), currency: acc["balances"]["iso_currency_code"].as_str().map(Into::into), balance_available: acc["balances"]["available"].as_f64(), balance_current: acc["balances"]["current"].as_f64(), institution_id: None, institution_name: None, backend: "plaid".into() })
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let resp = self.post("accounts/balance/get", serde_json::json!({})).await?;
        Ok(resp["accounts"].as_array().map(|a| a.iter().map(|acc| Balance { account_id: acc["account_id"].as_str().unwrap_or("").into(), account_name: acc["name"].as_str().map(Into::into), available: acc["balances"]["available"].as_f64(), current: acc["balances"]["current"].as_f64().unwrap_or(0.0), currency: acc["balances"]["iso_currency_code"].as_str().map(Into::into), backend: "plaid".into() }).collect()).unwrap_or_default())
    }

    async fn list_transactions(&self, account_id: &str, from: &str, to: &str, limit: u32) -> Result<Vec<Transaction>> {
        let resp = self.post("transactions/get", serde_json::json!({"start_date": from, "end_date": to, "options": {"account_ids": [account_id], "count": limit}})).await?;
        Ok(resp["transactions"].as_array().map(|a| a.iter().map(|t| Transaction { id: t["transaction_id"].as_str().unwrap_or("").into(), account_id: t["account_id"].as_str().unwrap_or("").into(), amount: t["amount"].as_f64().unwrap_or(0.0), currency: t["iso_currency_code"].as_str().map(Into::into), date: t["date"].as_str().unwrap_or("").into(), description: t["name"].as_str().unwrap_or("").into(), category: t["category"].as_array().and_then(|c| c.first()).and_then(|v| v.as_str()).map(Into::into), merchant_name: t["merchant_name"].as_str().map(Into::into), pending: t["pending"].as_bool().unwrap_or(false), transaction_type: t["transaction_type"].as_str().map(Into::into), backend: "plaid".into() }).collect()).unwrap_or_default())
    }

    async fn get_transaction(&self, id: &str) -> Result<Transaction> {
        let resp = self.post("transactions/get", serde_json::json!({"start_date": "2020-01-01", "end_date": "2030-12-31", "options": {"count": 500}})).await?;
        resp["transactions"].as_array().and_then(|a| a.iter().find(|t| t["transaction_id"].as_str() == Some(id))).map(|t| Transaction { id: t["transaction_id"].as_str().unwrap_or("").into(), account_id: t["account_id"].as_str().unwrap_or("").into(), amount: t["amount"].as_f64().unwrap_or(0.0), currency: t["iso_currency_code"].as_str().map(Into::into), date: t["date"].as_str().unwrap_or("").into(), description: t["name"].as_str().unwrap_or("").into(), category: t["category"].as_array().and_then(|c| c.first()).and_then(|v| v.as_str()).map(Into::into), merchant_name: t["merchant_name"].as_str().map(Into::into), pending: t["pending"].as_bool().unwrap_or(false), transaction_type: None, backend: "plaid".into() }).ok_or_else(|| anyhow::anyhow!("transaction not found"))
    }

    async fn search_transactions(&self, query: &str, limit: u32) -> Result<Vec<Transaction>> {
        let resp = self.post("transactions/get", serde_json::json!({"start_date": "2020-01-01", "end_date": "2030-12-31", "options": {"count": limit}})).await?;
        let q = query.to_lowercase();
        Ok(resp["transactions"].as_array().map(|a| a.iter().filter(|t| t["name"].as_str().unwrap_or("").to_lowercase().contains(&q) || t["merchant_name"].as_str().unwrap_or("").to_lowercase().contains(&q)).map(|t| Transaction { id: t["transaction_id"].as_str().unwrap_or("").into(), account_id: t["account_id"].as_str().unwrap_or("").into(), amount: t["amount"].as_f64().unwrap_or(0.0), currency: t["iso_currency_code"].as_str().map(Into::into), date: t["date"].as_str().unwrap_or("").into(), description: t["name"].as_str().unwrap_or("").into(), category: t["category"].as_array().and_then(|c| c.first()).and_then(|v| v.as_str()).map(Into::into), merchant_name: t["merchant_name"].as_str().map(Into::into), pending: t["pending"].as_bool().unwrap_or(false), transaction_type: None, backend: "plaid".into() }).collect()).unwrap_or_default())
    }

    async fn categorize_transaction(&self, id: &str) -> Result<Transaction> { self.get_transaction(id).await }

    async fn get_identity(&self) -> Result<Identity> {
        let resp = self.post("identity/get", serde_json::json!({})).await?;
        let owner = resp["accounts"].as_array().and_then(|a| a.first()).and_then(|acc| acc["owners"].as_array()).and_then(|o| o.first());
        Ok(Identity { name: owner.and_then(|o| o["names"].as_array()).and_then(|n| n.first()).and_then(|v| v.as_str()).map(Into::into), email: owner.and_then(|o| o["emails"].as_array()).and_then(|e| e.first()).and_then(|v| v["data"].as_str()).map(Into::into), phone: owner.and_then(|o| o["phone_numbers"].as_array()).and_then(|p| p.first()).and_then(|v| v["data"].as_str()).map(Into::into), address: owner.and_then(|o| o["addresses"].as_array()).and_then(|a| a.first()).and_then(|v| v["data"]["street"].as_str()).map(Into::into), backend: "plaid".into() })
    }

    async fn list_institutions(&self, country: Option<&str>, limit: u32) -> Result<Vec<Institution>> {
        let countries = country.map(|c| vec![c.to_uppercase()]).unwrap_or_else(|| vec!["US".into()]);
        let resp = self.post_no_token("institutions/get", serde_json::json!({"count": limit, "offset": 0, "country_codes": countries})).await?;
        Ok(resp["institutions"].as_array().map(|a| a.iter().map(|i| Institution { id: i["institution_id"].as_str().unwrap_or("").into(), name: i["name"].as_str().unwrap_or("").into(), country: i["country_codes"].as_array().and_then(|c| c.first()).and_then(|v| v.as_str()).map(Into::into), url: i["url"].as_str().map(Into::into), logo: i["logo"].as_str().map(Into::into), backend: "plaid".into() }).collect()).unwrap_or_default())
    }

    async fn get_institution(&self, id: &str) -> Result<Institution> {
        let resp = self.post_no_token("institutions/get_by_id", serde_json::json!({"institution_id": id, "country_codes": ["US", "GB", "EU"]})).await?;
        let i = &resp["institution"];
        Ok(Institution { id: i["institution_id"].as_str().unwrap_or("").into(), name: i["name"].as_str().unwrap_or("").into(), country: i["country_codes"].as_array().and_then(|c| c.first()).and_then(|v| v.as_str()).map(Into::into), url: i["url"].as_str().map(Into::into), logo: i["logo"].as_str().map(Into::into), backend: "plaid".into() })
    }

    async fn initiate_payment(&self, _account_id: &str, amount: f64, currency: &str, recipient: &str, reference: &str) -> Result<Payment> {
        let resp = self.post_no_token("payment_initiation/payment/create", serde_json::json!({"recipient_id": recipient, "reference": reference, "amount": {"value": amount, "currency": currency}})).await?;
        Ok(Payment { id: resp["payment_id"].as_str().unwrap_or("").into(), status: resp["status"].as_str().unwrap_or("created").into(), amount, currency: Some(currency.into()), recipient: Some(recipient.into()), reference: Some(reference.into()), created_at: None, backend: "plaid".into() })
    }

    async fn get_payment_status(&self, id: &str) -> Result<Payment> {
        let resp = self.post_no_token("payment_initiation/payment/get", serde_json::json!({"payment_id": id})).await?;
        Ok(Payment { id: resp["payment_id"].as_str().unwrap_or("").into(), status: resp["status"].as_str().unwrap_or("").into(), amount: resp["amount"]["value"].as_f64().unwrap_or(0.0), currency: resp["amount"]["currency"].as_str().map(Into::into), recipient: resp["recipient_id"].as_str().map(Into::into), reference: resp["reference"].as_str().map(Into::into), created_at: resp["created_at"].as_str().map(Into::into), backend: "plaid".into() })
    }

    async fn list_payments(&self, limit: u32) -> Result<Vec<Payment>> {
        let resp = self.post_no_token("payment_initiation/payment/list", serde_json::json!({"count": limit})).await?;
        Ok(resp["payments"].as_array().map(|a| a.iter().map(|p| Payment { id: p["payment_id"].as_str().unwrap_or("").into(), status: p["status"].as_str().unwrap_or("").into(), amount: p["amount"]["value"].as_f64().unwrap_or(0.0), currency: p["amount"]["currency"].as_str().map(Into::into), recipient: p["recipient_id"].as_str().map(Into::into), reference: p["reference"].as_str().map(Into::into), created_at: p["created_at"].as_str().map(Into::into), backend: "plaid".into() }).collect()).unwrap_or_default())
    }

    async fn get_statement(&self, account_id: &str, from: &str, to: &str) -> Result<Statement> {
        let txns = self.list_transactions(account_id, from, to, 500).await?;
        let balances = self.get_balances().await?;
        let current = balances.iter().find(|b| b.account_id == account_id).map(|b| b.current).unwrap_or(0.0);
        Ok(Statement { account_id: account_id.into(), period_start: from.into(), period_end: to.into(), opening_balance: None, closing_balance: Some(current), transaction_count: txns.len() as u32, backend: "plaid".into() })
    }

    async fn sync_transactions(&self) -> Result<String> {
        let resp = self.post("transactions/sync", serde_json::json!({})).await?;
        let added = resp["added"].as_array().map(|a| a.len()).unwrap_or(0);
        let modified = resp["modified"].as_array().map(|a| a.len()).unwrap_or(0);
        Ok(format!("Synced: {added} added, {modified} modified"))
    }
}
