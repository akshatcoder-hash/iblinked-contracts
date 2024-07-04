use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use crate::errors::ErrorCode;

pub fn fetch_pyth_price(
    price_update_info: &AccountInfo,
    feed_id: &str,
) -> Result<i64> {
    let clock = Clock::get()?;
    let feed_id = get_feed_id_from_hex(feed_id).map_err(|_| ErrorCode::PriceFetchFailed)?;

    let price_update = PriceUpdateV2::try_from_slice(&price_update_info.data.borrow())
        .map_err(|_| ErrorCode::PriceFetchFailed)?;
    
    const MAXIMUM_AGE: u64 = 60; // 60 seconds

    let price = price_update.get_price_no_older_than(&clock, MAXIMUM_AGE, &feed_id)
        .map_err(|_| ErrorCode::PriceFetchFailed)?;

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

pub fn calculate_refund_amount(original_amount: u64, elapsed_time: u64, total_duration: u64) -> u64 {
    if elapsed_time >= total_duration {
        return 0; // No refund if the market has ended
    }

    let time_ratio = (total_duration - elapsed_time) as f64 / total_duration as f64;
    let refund_ratio = time_ratio.powf(1.5); 

    (original_amount as f64 * refund_ratio) as u64
}