use anchor_lang::prelude::*;

use crate::constants::{MARKET_CREATION_AUTHORITY, MARKET_PDA_SEED, PRICE_FEED_CONFIG_PDA_SEED};
use crate::errors::ErrorCode;
use crate::state::{Market, PriceFeed, PriceFeedConfig};
use crate::utils::fetch_pyth_price;

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(
      mut,  
      seeds = [
        MARKET_PDA_SEED.as_bytes(), 
        MARKET_CREATION_AUTHORITY.as_ref(), 
        market.memecoin_symbol.as_bytes()
      ],
      bump,
      has_one = authority
    )]
    pub market: Account<'info, Market>,
    #[account(address = MARKET_CREATION_AUTHORITY)]
    pub authority: Signer<'info>,
    #[account(
      seeds = [
        PRICE_FEED_CONFIG_PDA_SEED.as_bytes(), 
        MARKET_CREATION_AUTHORITY.key().as_ref(), 
        price_feed.key().as_ref()
      ],
      bump
    )]
    pub price_feed_config: Account<'info, PriceFeedConfig>,
    #[account(address = price_feed_config.price_feed)]
    pub price_feed: Account<'info, PriceFeed>,
}

pub fn resolve_market(ctx: Context<ResolveMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let price_feed = &ctx.accounts.price_feed.to_account_info();

    // let current_time = Clock::get()?.unix_timestamp as u64;
    // if current_time <= market.start_time + market.duration {
    //     return Err(ErrorCode::MarketNotExpired.into());
    // }

    if market.resolved {
        return Err(ErrorCode::MarketAlreadyResolved.into());
    }

    let final_price = fetch_pyth_price(price_feed).map_err(|_| ErrorCode::PriceFetchFailed)?;

    let initial_price = market.initial_price.ok_or(ErrorCode::InitialPriceNotSet)?;

    market.resolved = true;
    market.winning_outcome = Some(final_price > initial_price);
    market.final_price = Some(final_price);

    Ok(())
}
