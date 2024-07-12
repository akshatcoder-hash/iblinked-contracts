use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod utils;

use crate::instructions::*;

declare_id!("ER8kXHjaGcktGNSXRtQqWzphrxsD3oPFRHoySryZfWx9");

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
    pub fn initialize_mock_pyth_feed(ctx: Context<InitializeMockPythFeed>, price: i64) -> Result<()> {
        ctx.accounts.price_feed.price = price;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeMockPythFeed<'info> {
    #[account(init, payer = payer, space = 8 + 8)]
    pub price_feed: Account<'info, MockPythFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct MockPythFeed {
    pub price: i64,
}
