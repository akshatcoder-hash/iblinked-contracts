use anchor_lang::prelude::*;

use crate::state::{Market, PriceFeed, PriceFeedConfig};
use crate::constants::{TEAM_WALLET, MARKET_PDA_SEED, MARKET_CREATION_FEE, MARKET_CREATION_AUTHORITY, PRICE_FEED_CONFIG_PDA_SEED};
use crate::utils::fetch_pyth_price;

#[derive(Accounts)]
#[instruction(memecoin_symbol: String)]
pub struct CreateMarket<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [
          MARKET_PDA_SEED.as_bytes(),
          authority.key().as_ref(), 
          memecoin_symbol.as_bytes()
        ],
        bump
    )]
    pub market: Account<'info, Market>,
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
    #[account(mut, address = MARKET_CREATION_AUTHORITY)]
    pub authority: Signer<'info>,
    #[account(mut, address = TEAM_WALLET)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub team_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_market(
  ctx: Context<CreateMarket>,
  memecoin_symbol: String,
  feed_id: String,
  duration: u64,
) -> Result<()> {
  let market = &mut ctx.accounts.market;
  let price_feed = &ctx.accounts.price_feed;

  let current_timestamp = Clock::get()?.unix_timestamp;
  let price = fetch_pyth_price(&price_feed.to_account_info())?;

  market.memecoin_symbol = memecoin_symbol;
  market.feed_id = feed_id;
  market.start_time = current_timestamp as u64;
  market.duration = duration;
  market.total_yes_shares = 0;
  market.total_no_shares = 0;
  market.resolved = false;
  market.winning_outcome = None;
  market.authority = ctx.accounts.authority.key();
  market.initial_price = Some(price);
  
  // set team fee unlock time (7 days after market resolution)
  market.team_fee_unlock_time = (market.start_time + market.duration + 7 * 24 * 60 * 60) as i64;

  let cpi_context = CpiContext::new(
      ctx.accounts.system_program.to_account_info(),
      anchor_lang::system_program::Transfer {
          from: ctx.accounts.authority.to_account_info(),
          to: ctx.accounts.team_wallet.to_account_info(),
      },
  );
  anchor_lang::system_program::transfer(cpi_context, MARKET_CREATION_FEE)?;

  Ok(())
}