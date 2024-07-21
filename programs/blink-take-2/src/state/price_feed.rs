use anchor_lang::prelude::*;

#[account]
pub struct PriceFeed {
    pub price: i64,
}
