use anchor_lang::prelude::*;

use crate::constants::{USER_POSITION_PDA_SEED, MARKET_PDA_SEED, MARKET_CREATION_AUTHORITY};
use crate::state::{Market, UserPosition};
use crate::utils::calculate_refund_amount;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct CancelBet<'info> {
    #[account(
      mut,
      seeds = [
        MARKET_PDA_SEED.as_bytes(), 
        MARKET_CREATION_AUTHORITY.as_ref(), 
        market.memecoin_symbol.as_bytes()
      ],
      bump
    )]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [USER_POSITION_PDA_SEED.as_bytes(), market.key().as_ref(), user.key().as_ref()],
        bump,
        has_one = market,
        has_one = user,
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;

    let current_time = Clock::get()?.unix_timestamp as u64;
    let elapsed_time = current_time - market.start_time;
    let six_hours_in_seconds = 6 * 60 * 60;

    if elapsed_time > six_hours_in_seconds {
        return Err(ErrorCode::MarketAlreadyStarted.into());
    }

    let total_shares = user_position.yes_shares + user_position.no_shares;

    // FIXME: is this correct? why is the 1.5 exponent thingy added?
    let refund_amount = calculate_refund_amount(total_shares, elapsed_time, market.duration);

    market.total_yes_shares -= user_position.yes_shares;
    market.total_no_shares -= user_position.no_shares;

    **market.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += refund_amount;

    user_position.yes_shares = 0;
    user_position.no_shares = 0;

    Ok(())
}
