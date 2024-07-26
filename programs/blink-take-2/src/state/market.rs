use anchor_lang::prelude::*;

#[account]
pub struct Market {
    pub memecoin_symbol: String,
    pub feed_id: String,
    pub start_time: u64,
    pub duration: u64,
    pub total_yes_shares: u64,
    pub total_no_shares: u64,
    pub resolved: bool,
    pub winning_outcome: Option<bool>,
    pub authority: Pubkey,
    pub initial_price: Option<i64>,
    pub final_price: Option<i64>,
    pub team_fee_paid: bool,
    pub team_fee_unlock_time: i64,
}
