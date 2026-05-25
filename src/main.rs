//! mcp-banking — Enterprise Banking MCP Server
mod types;
mod server;

#[cfg(feature = "plaid")]
mod plaid;
#[cfg(feature = "mono")]
mod mono;
#[cfg(feature = "open-banking")]
mod open_banking;

use rmcp::{ServiceExt, transport::stdio};
use server::BankingServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { tracing::error!("manifest: {e}"); }
        anyhow::bail!("invalid mcp-server.toml ({} error(s))", errors.len());
    }

    let backend: Arc<dyn types::BankingBackend> = init_backend()?;
    tracing::info!("{} v{} starting on stdio (backend: {})", manifest.display_name, manifest.version, backend.name());
    let server = BankingServer { backend };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

fn init_backend() -> anyhow::Result<Arc<dyn types::BankingBackend>> {
    #[cfg(feature = "plaid")]
    if let (Ok(client_id), Ok(secret), Ok(access_token)) = (std::env::var("PLAID_CLIENT_ID"), std::env::var("PLAID_SECRET"), std::env::var("PLAID_ACCESS_TOKEN")) {
        let env = std::env::var("PLAID_ENV").unwrap_or_else(|_| "sandbox".into());
        tracing::info!("Using Plaid backend ({env})");
        return Ok(Arc::new(plaid::PlaidBackend::new(client_id, secret, access_token, &env)));
    }

    #[cfg(feature = "mono")]
    if let (Ok(secret), Ok(account_id)) = (std::env::var("MONO_SECRET_KEY"), std::env::var("MONO_ACCOUNT_ID")) {
        tracing::info!("Using Mono (Africa) backend");
        return Ok(Arc::new(mono::MonoBackend::new(secret, account_id)));
    }

    #[cfg(feature = "open-banking")]
    if let (Ok(base_url), Ok(token)) = (std::env::var("OB_BASE_URL"), std::env::var("OB_TOKEN")) {
        tracing::info!("Using Open Banking backend");
        return Ok(Arc::new(open_banking::OpenBankingBackend::new(base_url, token)));
    }

    anyhow::bail!("No banking backend configured. Set one of: PLAID_CLIENT_ID+PLAID_SECRET+PLAID_ACCESS_TOKEN, MONO_SECRET_KEY+MONO_ACCOUNT_ID, OB_BASE_URL+OB_TOKEN")
}
