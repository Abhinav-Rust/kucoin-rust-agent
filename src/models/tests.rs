use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_config_default_fallbacks() {
        std::env::remove_var("MARGIN_USDT");
        std::env::remove_var("LEVERAGE");
        std::env::remove_var("PROFIT_TARGET_USDT");
        std::env::remove_var("SYMBOL");

        let config = PositionConfig::from_env();

        assert_eq!(config.margin_usdt, DEFAULT_MARGIN_USDT);
        assert_eq!(config.leverage, DEFAULT_LEVERAGE);
        assert_eq!(config.profit_target_usdt, DEFAULT_PROFIT_TARGET_USDT);
        assert_eq!(config.symbol, DEFAULT_SYMBOL);
    }

    #[test]
    fn test_contract_response_deserialization() {
        let json_data = r#"{
            "data": {
                "lastTradePrice": 145.5,
                "multiplier": 0.1,
                "tickSize": 0.001
            }
        }"#;

        let response: ContractResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(response.data.last_trade_price, 145.5);
        assert_eq!(response.data.multiplier, 0.1);
        assert_eq!(response.data.tick_size, 0.001);
    }
}
