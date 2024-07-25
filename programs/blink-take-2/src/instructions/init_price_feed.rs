use anchor_lang::prelude::*;

use crate::constants::MARKET_CREATION_AUTHORITY;
use crate::constants::PRICE_FEED_CONFIG_PDA_SEED;
use crate::state::PriceFeedConfig;

#[derive(Accounts)]
#[instruction(feed: Pubkey)]
pub struct InitializePriceFeed<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<PriceFeedConfig>(),
        seeds = [
          PRICE_FEED_CONFIG_PDA_SEED.as_bytes(), 
          payer.key().as_ref(), 
          feed.key().as_ref()
        ],
        bump
    )]
    pub price_feed_config: Account<'info, PriceFeedConfig>,
    #[account(mut, address = MARKET_CREATION_AUTHORITY)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_price_feed(ctx: Context<InitializePriceFeed>, feed: Pubkey) -> Result<()> {
    ctx.accounts.price_feed_config.price_feed = feed;

    Ok(())
}
