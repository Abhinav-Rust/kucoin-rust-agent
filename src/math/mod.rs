use crate::models::{PositionCalculation, PositionConfig};

/// Dynamically calculates the exact position size (in lots) and take-profit exit price.
///
/// It uses the `last_trade_price` and `multiplier` to convert the leveraged USDT position value
/// into contract lots, ensuring proper integer bounds. It then calculates the necessary price
/// shift (`tp_price_diff`) for the target profit, ensuring the result is properly aligned with
/// the required `tick_size` step.
pub fn calculate_position(
    config: &PositionConfig,
    last_trade_price: f64,
    multiplier: f64,
    tick_size: f64,
) -> Option<PositionCalculation> {
    let position_value_usdt = config.margin_usdt * config.leverage;
    let position_size_sol = position_value_usdt / last_trade_price;

    // Calculate raw lots (contract size)
    let raw_lots = position_size_sol / multiplier;

    // Round to nearest integer lot size
    let lots = raw_lots.round() as i64;

    if lots <= 0 {
        return None;
    }

    // Calculate target take profit price
    // Profit = (TP - Entry) * (Lots * Multiplier)
    let tp_price_diff = config.profit_target_usdt / (lots as f64 * multiplier);
    let target_tp_price = last_trade_price + tp_price_diff;

    // Round to nearest tick_size
    let precision = tick_size_precision(tick_size);
    let rounded_tp_price = format_price(target_tp_price, tick_size, precision);

    Some(PositionCalculation {
        position_value_usdt,
        lots,
        target_tp_price: rounded_tp_price,
    })
}

/// Calculates the required decimal places from a tick size (e.g. 0.001 -> 3, 0.1 -> 1, 1 -> 0).
pub fn tick_size_precision(tick_size: f64) -> usize {
    if tick_size <= 0.0 {
        return 2;
    }
    let s = format!("{:.8}", tick_size);
    let s = s.trim_end_matches('0');
    if let Some(pos) = s.find('.') {
        s[pos + 1..].len()
    } else {
        0
    }
}

/// Rounds a price to the nearest tick step and formats/parses it to remove floating point artifacts.
pub fn format_price(price: f64, tick_size: f64, precision: usize) -> f64 {
    let rounded = (price / tick_size).round() * tick_size;
    let formatted = format!("{:.1$}", rounded, precision);
    formatted.parse::<f64>().unwrap_or(rounded)
}

#[cfg(test)]
mod tests;
