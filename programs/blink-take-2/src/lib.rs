use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use crate::instructions::*;

declare_id!("HiXkrawYru9nPuyddUTD83xWjRb236r3NVLp67EeuGSw");

#[program]
pub mod blink_take_2 {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        memecoin_symbol: String,
        feed_id: String,
        duration: u64,
    ) -> Result<()> {
        instructions::create_market(ctx, memecoin_symbol, feed_id, duration)
    }

    pub fn create_user(ctx: Context<CreateUser>) -> Result<()> {
        instructions::create_user(ctx)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, choice: bool) -> Result<()> {
        instructions::place_bet(ctx, amount, choice)
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>) -> Result<()> {
        instructions::resolve_market(ctx)
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        instructions::claim_winnings(ctx)
    }

    pub fn withdraw_team_fee(ctx: Context<WithdrawTeamFee>) -> Result<()> {
        instructions::withdraw_team_fee(ctx)
    }

    pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
        instructions::cancel_bet(ctx)
    }

    pub fn initialize_price_feed(ctx: Context<InitializePriceFeed>, feed: Pubkey) -> Result<()> {
        instructions::initialize_price_feed(ctx, feed)
    }
}
