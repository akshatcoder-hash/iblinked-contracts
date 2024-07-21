use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use pyth_sdk_solana::state::SolanaPriceAccount;

pub fn fetch_pyth_price(price_feed_info: &AccountInfo) -> Result<i64> {
    let price_feed = SolanaPriceAccount::account_info_to_feed(price_feed_info)
        .map_err(|_| ErrorCode::PriceFetchFailed)?;

    let price = price_feed.get_price_unchecked();
    Ok(price.price)
}

pub fn validate_active_period(start_time: u64, duration: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp as u64;
    let end_time = start_time + duration;

    if current_time < start_time || current_time > end_time {
        return Err(ErrorCode::MarketNotActive.into());
    }

    Ok(())
}

pub fn calculate_shares(amount: u64) -> u64 {
    const BASE: f64 = 1_000_000.0;
    const EXPONENT: f64 = 1.1;

    ((amount as f64 / BASE).powf(EXPONENT) * BASE) as u64
}

pub fn calculate_refund_amount(
    original_amount: u64,
    elapsed_time: u64,
    total_duration: u64,
) -> u64 {
    if elapsed_time >= total_duration {
        return 0;
    }

    let time_ratio = (total_duration - elapsed_time) as f64 / total_duration as f64;
    let refund_ratio = time_ratio.powf(1.5);

    (original_amount as f64 * refund_ratio) as u64
}
