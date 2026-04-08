use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use reqwest::{header::{HeaderMap, HeaderValue}, Client};
use serde_json::json;
use sha2::Sha256;
use std::{env, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const API_BASE_URL: &str = "https://api-futures.kucoin.com";
const MARGIN_USDT: f64 = 13.0;
const LEVERAGE: f64 = 20.0;
const PROFIT_TARGET_USDT: f64 = 1.5;
const SYMBOL: &str = "SOLUSDTM";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load variables from .env
    dotenv::dotenv().ok();
    
    let api_key = env::var("KUCOIN_API_KEY").expect("KUCOIN_API_KEY must be set");
    let api_secret = env::var("KUCOIN_API_SECRET").expect("KUCOIN_API_SECRET must be set");
    let api_passphrase = env::var("KUCOIN_API_PASSPHRASE").expect("KUCOIN_API_PASSPHRASE must be set");

    let client = Client::new();

    println!("1. Fetching contract details for {}", SYMBOL);
    let contract_endpoint = format!("/api/v1/contracts/{}", SYMBOL);
    let headers = generate_auth_headers("GET", &contract_endpoint, "", &api_key, &api_secret, &api_passphrase);
    
    let url = format!("{}{}", API_BASE_URL, contract_endpoint);
    let res = client.get(&url).headers(headers).send().await?;
    
    if !res.status().is_success() {
        let error_text = res.text().await?;
        eprintln!("Failed to fetch contract details: {}", error_text);
        return Ok(());
    }
    
    let body: serde_json::Value = res.json().await?;
    let data = &body["data"];
    
    let last_trade_price = data["lastTradePrice"].as_f64().expect("Failed to parse lastTradePrice");
    let multiplier = data["multiplier"].as_f64().expect("Failed to parse multiplier");
    let tick_size = data["tickSize"].as_f64().expect("Failed to parse tickSize");
    
    println!("Current Price: USDT {}", last_trade_price);
    println!("Contract Multiplier: {}", multiplier);
    println!("Tick Size: {}", tick_size);

    // Calculate position size
    let position_value_usdt = MARGIN_USDT * LEVERAGE;
    let position_size_sol = position_value_usdt / last_trade_price;
    // Calculate raw lots (contract size)
    let raw_lots = position_size_sol / multiplier;
    // Round to nearest integer lot size
    let lots = raw_lots.round() as i64;

    if lots <= 0 {
        eprintln!("Calculated contract size is 0. Not enough margin or price is too high.");
        return Ok(());
    }

    println!("Position Value: USDT {}", position_value_usdt);
    println!("Calculated Lots: {}", lots);

    // Calculate target take profit price
    // Profit = (TP - Entry) * (Lots * Multiplier)
    let tp_price_diff = PROFIT_TARGET_USDT / (lots as f64 * multiplier);
    let target_tp_price = last_trade_price + tp_price_diff;
    // Round to nearest tick_size
    let rounded_tp_price = (target_tp_price / tick_size).round() * tick_size;

    println!("Target Take Profit Price: {}", rounded_tp_price);

    println!("2. Executing Market Long Entry");
    let orders_endpoint = "/api/v1/orders";
    let entry_body = json!({
        "clientOid": Uuid::new_v4().to_string(),
        "symbol": SYMBOL,
        "side": "buy",
        "leverage": LEVERAGE as i64,
        "type": "market",
        "size": lots.to_string(),
        "marginMode": "ISOLATED"
    }).to_string();

    let entry_headers = generate_auth_headers("POST", orders_endpoint, &entry_body, &api_key, &api_secret, &api_passphrase);
    let entry_url = format!("{}{}", API_BASE_URL, orders_endpoint);
    
    let entry_res = client.post(&entry_url)
        .headers(entry_headers)
        .header("Content-Type", "application/json")
        .body(entry_body)
        .send().await?;

    println!("Entry Order HTTP Status: {}", entry_res.status());
    let entry_res_text = entry_res.text().await?;
    println!("Entry Response: {}", entry_res_text);

    println!("3. Placing Limit Take Profit Order");
    let tp_body = json!({
        "clientOid": Uuid::new_v4().to_string(),
        "symbol": SYMBOL,
        "side": "sell",
        "leverage": LEVERAGE as i64,
        "type": "limit",
        "size": lots.to_string(),
        "price": format!("{:.3}", rounded_tp_price),
        "closeOrder": true,
        "marginMode": "ISOLATED"
    }).to_string();

    let tp_headers = generate_auth_headers("POST", orders_endpoint, &tp_body, &api_key, &api_secret, &api_passphrase);
    
    let tp_res = client.post(&entry_url)
        .headers(tp_headers)
        .header("Content-Type", "application/json")
        .body(tp_body)
        .send().await?;

    println!("Take Profit Order HTTP Status: {}", tp_res.status());
    let tp_res_text = tp_res.text().await?;
    println!("Take Profit Response: {}", tp_res_text);

    println!("Strategy execution completed seamlessly!");

    Ok(())
}

fn generate_auth_headers(
    method: &str,
    endpoint: &str,
    body: &str,
    api_key: &str,
    api_secret: &str,
    api_passphrase: &str,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let str_to_sign = format!("{}{}{}{}", timestamp, method, endpoint, body);
    
    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes()).unwrap();
    mac.update(str_to_sign.as_bytes());
    let signature = BASE64.encode(mac.finalize().into_bytes());

    let mut pass_mac = HmacSha256::new_from_slice(api_secret.as_bytes()).unwrap();
    pass_mac.update(api_passphrase.as_bytes());
    let pass_signature = BASE64.encode(pass_mac.finalize().into_bytes());

    headers.insert("KC-API-KEY", HeaderValue::from_str(api_key).unwrap());
    headers.insert("KC-API-SIGN", HeaderValue::from_str(&signature).unwrap());
    headers.insert("KC-API-TIMESTAMP", HeaderValue::from_str(&timestamp).unwrap());
    headers.insert("KC-API-PASSPHRASE", HeaderValue::from_str(&pass_signature).unwrap());
    headers.insert("KC-API-KEY-VERSION", HeaderValue::from_static("2"));

    headers
}
