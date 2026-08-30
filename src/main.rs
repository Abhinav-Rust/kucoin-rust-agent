pub mod api;
pub mod auth;
pub mod math;
pub mod models;

use anyhow::{Context, Result};
use api::KuCoinApiClient;
use auth::Credentials;
use models::PositionConfig;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Load variables from .env
    dotenv::dotenv().ok();

    let api_key = env::var("KUCOIN_API_KEY").context("KUCOIN_API_KEY must be set in .env")?;
    let api_secret =
        env::var("KUCOIN_API_SECRET").context("KUCOIN_API_SECRET must be set in .env")?;
    let api_passphrase =
        env::var("KUCOIN_API_PASSPHRASE").context("KUCOIN_API_PASSPHRASE must be set in .env")?;

    let credentials = Credentials {
        api_key,
        api_secret,
        api_passphrase,
    };

    let config = PositionConfig::from_env();

    let client = KuCoinApiClient::new(credentials);

    println!("1. Fetching contract details for {}", config.symbol);
    let contract_response = client.fetch_contract_details(&config.symbol).await?;
    let data = contract_response.data;

    println!("Current Price: USDT {}", data.last_trade_price);
    println!("Contract Multiplier: {}", data.multiplier);
    println!("Tick Size: {}", data.tick_size);

    let calc = math::calculate_position(
        &config,
        data.last_trade_price,
        data.multiplier,
        data.tick_size,
    )
    .context("Calculated contract size is 0. Not enough margin or price is too high.")?;

    println!("Position Value: USDT {}", calc.position_value_usdt);
    println!("Calculated Lots: {}", calc.lots);
    println!("Target Take Profit Price: {}", calc.target_tp_price);

    println!("2. Executing Market Long Entry");
    let entry_res = client.place_market_entry(&config, calc.lots).await?;
    println!("Entry Order HTTP Status: {}", entry_res.status());

    if !entry_res.status().is_success() {
        let entry_res_text = entry_res.text().await?;
        anyhow::bail!(
            "Market entry order failed. Aborting to prevent naked orders. Response: {}",
            entry_res_text
        );
    }

    let entry_res_text = entry_res.text().await?;
    println!("Entry Response: {}", entry_res_text);

    println!("3. Placing Limit Take Profit Order");
    let tp_res = client
        .place_limit_take_profit(&config, calc.lots, calc.target_tp_price, data.tick_size)
        .await?;
    println!("Take Profit Order HTTP Status: {}", tp_res.status());

    if !tp_res.status().is_success() {
        let tp_res_text = tp_res.text().await?;
        anyhow::bail!(
            "Take profit order failed. You may have a naked position! Response: {}",
            tp_res_text
        );
    }

    let tp_res_text = tp_res.text().await?;
    println!("Take Profit Response: {}", tp_res_text);

    println!("Strategy execution completed seamlessly!");

    Ok(())
}
