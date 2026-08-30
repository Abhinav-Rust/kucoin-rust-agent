use serde::Deserialize;

/// Represents the raw contract data received from KuCoin's API.
#[derive(Debug, Deserialize)]
pub struct ContractData {
    /// The most recent price the contract traded at.
    #[serde(rename = "lastTradePrice")]
    pub last_trade_price: f64,
    /// The contract multiplier used to convert between contract lots and underlying asset.
    pub multiplier: f64,
    /// The minimum price change (tick size) allowed for the contract.
    #[serde(rename = "tickSize")]
    pub tick_size: f64,
}

/// Standard response wrapper from KuCoin API.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub msg: Option<String>,
    pub data: Option<T>,
}

/// A wrapper for the successful response payload containing contract data.
#[derive(Debug, Deserialize)]
pub struct ContractResponse {
    pub code: Option<String>,
    pub msg: Option<String>,
    pub data: ContractData,
}

/// Default fallback constants for position configuration.
pub const DEFAULT_MARGIN_USDT: f64 = 13.0;
pub const DEFAULT_LEVERAGE: f64 = 20.0;
pub const DEFAULT_PROFIT_TARGET_USDT: f64 = 1.5;
pub const DEFAULT_SYMBOL: &str = "SOLUSDTM";

/// Configuration settings for calculating position sizes and targets.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionConfig {
    /// The capital allocated to this trade in USDT.
    pub margin_usdt: f64,
    /// The leverage applied to the position.
    pub leverage: f64,
    /// The desired profit target in USDT.
    pub profit_target_usdt: f64,
    /// The trading pair symbol (e.g., "SOLUSDTM").
    pub symbol: String,
}

impl PositionConfig {
    /// Loads configuration from environment variables with fallback defaults.
    pub fn from_env() -> Self {
        let margin_usdt = std::env::var("MARGIN_USDT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_MARGIN_USDT);

        let leverage = std::env::var("LEVERAGE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_LEVERAGE);

        let profit_target_usdt = std::env::var("PROFIT_TARGET_USDT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_PROFIT_TARGET_USDT);

        let symbol = std::env::var("SYMBOL").unwrap_or_else(|_| DEFAULT_SYMBOL.to_string());

        Self {
            margin_usdt,
            leverage,
            profit_target_usdt,
            symbol,
        }
    }
}

/// The result of calculating order parameters from live market data.
#[derive(Debug)]
pub struct PositionCalculation {
    /// The total value of the position in USDT (Margin * Leverage).
    pub position_value_usdt: f64,
    /// The calculated number of contracts (lots) to trade.
    pub lots: i64,
    /// The exact target price for the take-profit order, rounded to the tick size.
    pub target_tp_price: f64,
}

#[cfg(test)]
mod tests;
