use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_position_success() {
        let config = PositionConfig {
            margin_usdt: 13.0,
            leverage: 20.0,
            profit_target_usdt: 1.5,
            symbol: "SOLUSDTM".to_string(),
        };

        // Simulated market data
        let last_trade_price = 150.0;
        let multiplier = 0.1;
        let tick_size = 0.001;

        let result = calculate_position(&config, last_trade_price, multiplier, tick_size);

        assert!(result.is_some());
        let calc = result.unwrap();

        // Position value = 13 * 20 = 260
        assert_eq!(calc.position_value_usdt, 260.0);

        // Raw lots = 260 / 150 / 0.1 = 17.333...
        // Rounded lots = 17
        assert_eq!(calc.lots, 17);

        // TP Price Diff = 1.5 / (17 * 0.1) = 1.5 / 1.7 = 0.88235...
        // Target TP Price = 150.0 + 0.88235... = 150.88235...
        // Rounded to tick (0.001) = 150.882
        assert_eq!(calc.target_tp_price, 150.882);
    }

    #[test]
    fn test_calculate_position_not_enough_margin() {
        let config = PositionConfig {
            margin_usdt: 1.0,
            leverage: 1.0,
            profit_target_usdt: 1.5,
            symbol: "BTCUSDTM".to_string(),
        };

        // Price too high, margin too low
        let last_trade_price = 60000.0;
        let multiplier = 0.001;
        let tick_size = 0.1;

        // Position value = 1.0
        // Size in BTC = 1.0 / 60000.0 = 0.0000166...
        // Raw lots = 0.0000166... / 0.001 = 0.0166...
        // Rounded lots = 0
        let result = calculate_position(&config, last_trade_price, multiplier, tick_size);

        assert!(result.is_none());
    }

    #[test]
    fn test_tick_size_precision() {
        assert_eq!(tick_size_precision(0.001), 3);
        assert_eq!(tick_size_precision(0.1), 1);
        assert_eq!(tick_size_precision(1.0), 0);
        assert_eq!(tick_size_precision(0.00001), 5);
        assert_eq!(tick_size_precision(0.0), 2);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(150.88235, 0.001, 3), 150.882);
        assert_eq!(format_price(65432.14, 0.1, 1), 65432.1);
        assert_eq!(format_price(100.0, 1.0, 0), 100.0);
    }
}
