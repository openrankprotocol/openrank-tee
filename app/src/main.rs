use alloy::hex::FromHex;
use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use alloy::rpc::client::RpcClient;
use alloy::signers::local::coins_bip39::English;
use alloy::signers::local::MnemonicBuilder;
use alloy::transports::http::reqwest::Url;
use aws_config::from_env;
use aws_sdk_s3::Client;
use clap::Parser;
use dotenv::dotenv;
use openrank_app::sol::OpenRankManager;
use openrank_app::{challenger, computer};
use openrank_common::logs::setup_tracing;

const BUCKET_NAME: &str = "openrank-data-dev";
const BLOCK_HISTORY: u64 = 100;
const LOG_PULL_INTERVAL_SECONDS: u64 = 10;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    challenger: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    dotenv().ok();
    setup_tracing();

    let cli = Args::parse();

    let rpc_url = std::env::var("CHAIN_RPC_URL").expect("CHAIN_RPC_URL must be set.");
    let manager_address =
        std::env::var("OPENRANK_MANAGER_ADDRESS").expect("OPENRANK_MANAGER_ADDRESS must be set.");
    let mnemonic = std::env::var("MNEMONIC").expect("MNEMONIC must be set.");
    let config = from_env().region("us-west-2").load().await;
    let client = Client::new(&config);

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .index(0)
        .map_err(|e| format!("Failed to set mnemonic index: {}", e))?
        .build()
        .map_err(|e| format!("Failed to build wallet: {}", e))?;

    let rpc_url_parsed = Url::parse(&rpc_url)
        .map_err(|e| format!("Failed to parse RPC URL '{}': {}", rpc_url, e))?;
    let provider_http = ProviderBuilder::new()
        .wallet(wallet.clone())
        .connect_client(RpcClient::new_http(rpc_url_parsed));

    let manager_address = Address::from_hex(manager_address)
        .map_err(|e| format!("Failed to parse manager address: {}", e))?;
    let manager_contract = OpenRankManager::new(manager_address, provider_http.clone());

    if cli.challenger {
        if let Err(e) = challenger::run(
            manager_contract,
            provider_http.clone(),
            client,
            BUCKET_NAME,
            BLOCK_HISTORY,
            LOG_PULL_INTERVAL_SECONDS,
        )
        .await
        {
            eprintln!("Challenger failed: {}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = computer::run(
            manager_contract,
            provider_http,
            client,
            BUCKET_NAME,
            BLOCK_HISTORY,
            LOG_PULL_INTERVAL_SECONDS,
        )
        .await
        {
            eprintln!("Computer failed: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
