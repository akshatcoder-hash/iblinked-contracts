use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::Market;
use crate::utils::fetch_pyth_price;

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
    pub authority: Signer<'info>,
    /// CHECK: This account is checked in the instruction
    pub price_feed: AccountInfo<'info>,
}

pub fn resolve_market(ctx: Context<ResolveMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let price_feed = &ctx.accounts.price_feed;

    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time <= market.start_time + market.duration {
        return Err(ErrorCode::MarketNotExpired.into());
    }

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
