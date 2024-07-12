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
    pub total_funds: u64,
    pub authority: Pubkey,
    pub initial_price: Option<i64>,
    pub final_price: Option<i64>,
    pub team_fee_paid: bool,
    pub team_fee_unlock_time: i64,
}

#[account]
pub struct UserPosition {
    pub market: Pubkey,
    pub user: Pubkey,
    pub yes_shares: u64,
    pub no_shares: u64,
    pub claimed: bool,
}

#[account]
pub struct PythFeedAccount {
    pub price: i64,
    pub conf: u64,
    pub status: u32,
    pub pub_slot: u64,
}