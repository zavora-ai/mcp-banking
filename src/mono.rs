//! Mono (Africa) API backend — Nigeria, Kenya, Ghana, South Africa.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.withmono.com/v2";

#[derive(Clone)]
pub struct MonoBackend { http: Client, secret_key: String, account_id: String }

impl MonoBackend {
    pub fn new(secret_key: String, account_id: String) -> Self {
        Self { http: Client::new(), secret_key, account_id }
    }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("mono-sec-key", &self.secret_key).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("mono-sec-key", &self.secret_key).json(body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl BankingBackend for MonoBackend {
    fn name(&self) -> &str { "mono" }

    async fn list_accounts(&self) -> Result<Vec<BankAccount>> {
        let acc = self.get_account(&self.account_id).await?;
        Ok(vec![acc])
    }

    async fn get_account(&self, id: &str) -> Result<BankAccount> {
        let resp = self.get(&format!("accounts/{id}")).await?;
        let d = &resp["data"];
        Ok(BankAccount { id: id.into(), name: d["name"].as_str().unwrap_or("").into(), account_type: d["type"].as_str().map(Into::into), subtype: None, mask: d["account_number"].as_str().map(|s| s[s.len().saturating_sub(4)..].to_string()), currency: d["currency"].as_str().map(Into::into), balance_available: None, balance_current: d["balance"].as_f64().map(|b| b / 100.0), institution_id: None, institution_name: d["institution"]["name"].as_str().map(Into::into), backend: "mono".into() })
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let acc = self.get_account(&self.account_id).await?;
        Ok(vec![Balance { account_id: acc.id, account_name: Some(acc.name), available: acc.balance_available, current: acc.balance_current.unwrap_or(0.0), currency: acc.currency, backend: "mono".into() }])
    }

    async fn list_transactions(&self, _account_id: &str, from: &str, to: &str, limit: u32) -> Result<Vec<Transaction>> {
        let resp = self.get(&format!("accounts/{}/transactions?start={}&end={}&limit={}", self.account_id, from, to, limit)).await?;
        Ok(resp["data"].as_array().map(|a| a.iter().map(|t| Transaction { id: t["_id"].as_str().unwrap_or("").into(), account_id: self.account_id.clone(), amount: t["amount"].as_f64().map(|a| a / 100.0).unwrap_or(0.0), currency: None, date: t["date"].as_str().unwrap_or("").into(), description: t["narration"].as_str().unwrap_or("").into(), category: t["category"].as_str().map(Into::into), merchant_name: None, pending: false, transaction_type: t["type"].as_str().map(Into::into), backend: "mono".into() }).collect()).unwrap_or_default())
    }

    async fn get_transaction(&self, id: &str) -> Result<Transaction> {
        let resp = self.get(&format!("accounts/{}/transactions/{}", self.account_id, id)).await?;
        let t = &resp["data"];
        Ok(Transaction { id: t["_id"].as_str().unwrap_or("").into(), account_id: self.account_id.clone(), amount: t["amount"].as_f64().map(|a| a / 100.0).unwrap_or(0.0), currency: None, date: t["date"].as_str().unwrap_or("").into(), description: t["narration"].as_str().unwrap_or("").into(), category: t["category"].as_str().map(Into::into), merchant_name: None, pending: false, transaction_type: t["type"].as_str().map(Into::into), backend: "mono".into() })
    }

    async fn search_transactions(&self, query: &str, limit: u32) -> Result<Vec<Transaction>> {
        let txns = self.list_transactions(&self.account_id, "2020-01-01", "2030-12-31", limit).await?;
        let q = query.to_lowercase();
        Ok(txns.into_iter().filter(|t| t.description.to_lowercase().contains(&q)).collect())
    }

    async fn categorize_transaction(&self, id: &str) -> Result<Transaction> { self.get_transaction(id).await }

    async fn get_identity(&self) -> Result<Identity> {
        let resp = self.get(&format!("accounts/{}/identity", self.account_id)).await?;
        let d = &resp["data"];
        Ok(Identity { name: d["full_name"].as_str().map(Into::into), email: d["email"].as_str().map(Into::into), phone: d["phone"].as_str().map(Into::into), address: d["address_line1"].as_str().map(Into::into), backend: "mono".into() })
    }

    async fn list_institutions(&self, country: Option<&str>, _limit: u32) -> Result<Vec<Institution>> {
        let path = match country {
            Some(c) => format!("coverage?country={c}"),
            None => "coverage".into(),
        };
        let resp = self.get(&path).await?;
        Ok(resp["data"].as_array().map(|a| a.iter().map(|i| Institution { id: i["_id"].as_str().unwrap_or("").into(), name: i["name"].as_str().unwrap_or("").into(), country: i["country"].as_str().map(Into::into), url: None, logo: i["icon"].as_str().map(Into::into), backend: "mono".into() }).collect()).unwrap_or_default())
    }

    async fn get_institution(&self, id: &str) -> Result<Institution> {
        let insts = self.list_institutions(None, 100).await?;
        insts.into_iter().find(|i| i.id == id).ok_or_else(|| anyhow::anyhow!("institution not found"))
    }

    async fn initiate_payment(&self, _account_id: &str, amount: f64, _currency: &str, recipient: &str, reference: &str) -> Result<Payment> {
        let resp = self.post("payments/initiate", &serde_json::json!({"amount": (amount * 100.0) as i64, "type": "onetime", "description": reference, "account": recipient})).await?;
        Ok(Payment { id: resp["data"]["id"].as_str().unwrap_or("").into(), status: resp["data"]["status"].as_str().unwrap_or("pending").into(), amount, currency: None, recipient: Some(recipient.into()), reference: Some(reference.into()), created_at: None, backend: "mono".into() })
    }

    async fn get_payment_status(&self, id: &str) -> Result<Payment> {
        let resp = self.get(&format!("payments/{id}")).await?;
        let d = &resp["data"];
        Ok(Payment { id: d["id"].as_str().unwrap_or("").into(), status: d["status"].as_str().unwrap_or("").into(), amount: d["amount"].as_f64().map(|a| a / 100.0).unwrap_or(0.0), currency: None, recipient: d["account"].as_str().map(Into::into), reference: d["description"].as_str().map(Into::into), created_at: d["created_at"].as_str().map(Into::into), backend: "mono".into() })
    }

    async fn list_payments(&self, _limit: u32) -> Result<Vec<Payment>> {
        let resp = self.get("payments").await?;
        Ok(resp["data"].as_array().map(|a| a.iter().map(|p| Payment { id: p["id"].as_str().unwrap_or("").into(), status: p["status"].as_str().unwrap_or("").into(), amount: p["amount"].as_f64().map(|a| a / 100.0).unwrap_or(0.0), currency: None, recipient: p["account"].as_str().map(Into::into), reference: p["description"].as_str().map(Into::into), created_at: p["created_at"].as_str().map(Into::into), backend: "mono".into() }).collect()).unwrap_or_default())
    }

    async fn get_statement(&self, _account_id: &str, from: &str, to: &str) -> Result<Statement> {
        let resp = self.post(&format!("accounts/{}/statement", self.account_id), &serde_json::json!({"period": format!("{from}_{to}")})).await?;
        let d = &resp["data"];
        Ok(Statement { account_id: self.account_id.clone(), period_start: from.into(), period_end: to.into(), opening_balance: d["opening_balance"].as_f64().map(|b| b / 100.0), closing_balance: d["closing_balance"].as_f64().map(|b| b / 100.0), transaction_count: d["transactions"].as_array().map(|a| a.len() as u32).unwrap_or(0), backend: "mono".into() })
    }

    async fn sync_transactions(&self) -> Result<String> {
        self.post(&format!("accounts/{}/sync", self.account_id), &serde_json::json!({})).await?;
        Ok("Sync triggered".into())
    }
}
