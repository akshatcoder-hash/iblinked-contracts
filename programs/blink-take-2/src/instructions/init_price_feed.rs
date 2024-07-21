use anchor_lang::prelude::*;

use crate::constants::MARKET_CREATION_AUTHORITY;
use crate::constants::PRICE_FEED_PDA_SEED;
use crate::state::PriceFeed;

#[derive(Accounts)]
#[instruction(price: i64)]
pub struct InitializePriceFeed<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<PriceFeed>(),
        seeds = [PRICE_FEED_PDA_SEED.as_bytes(), payer.key().as_ref()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    #[account(mut, address = MARKET_CREATION_AUTHORITY)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_price_feed(ctx: Context<InitializePriceFeed>, price: i64) -> Result<()> {
    ctx.accounts.price_feed.price = price;

    Ok(())
}
