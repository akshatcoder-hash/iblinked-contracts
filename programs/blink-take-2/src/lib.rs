use anchor_lang::prelude::*;
// use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod utils;

use crate::instructions::*;

declare_id!("6kFaTqPvBH631tcQtQMfAYbpwxRe3e1bUrpy6Wo2aQyj");

pub const MARKET_CREATION_FEE: u64 = 100_000_000; // 0.1 SOL

#[program]
pub mod prediction_blink {
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
}