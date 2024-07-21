use anchor_lang::prelude::*;

use crate::constants::MARKET_CREATION_AUTHORITY;
use crate::state::PriceFeed;

#[derive(Accounts)]
pub struct UpdatePriceFeed<'info> {
    #[account(mut)]
    pub price_feed: Account<'info, PriceFeed>,
    #[account(address = MARKET_CREATION_AUTHORITY)]
    pub signer: Signer<'info>,
}

pub fn update_price_feed(ctx: Context<UpdatePriceFeed>, price: i64) -> Result<()> {
    let price_feed = &mut ctx.accounts.price_feed;

    price_feed.price = price;

    Ok(())
}
