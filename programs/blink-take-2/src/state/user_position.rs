use anchor_lang::prelude::*;

#[account]
pub struct UserPosition {
    pub market: Pubkey,
    pub user: Pubkey,
    pub yes_shares: u64,
    pub no_shares: u64,
    pub claimed: bool,
}
