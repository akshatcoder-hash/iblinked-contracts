use anchor_lang::prelude::*;

use crate::constants::{MIN_BET_AMOUNT, USER_POSITION_PDA_SEED};
use crate::errors::ErrorCode;
use crate::state::{Market, UserPosition};
use crate::utils::calculate_shares;

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [USER_POSITION_PDA_SEED.as_bytes(), market.key().as_ref(), user.key().as_ref()],
        bump,
        has_one = market,
        has_one = user
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, choice: bool) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;

    if amount < MIN_BET_AMOUNT {
        return Err(ErrorCode::BetAmountTooLow.into());
    }

    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time < market.start_time || current_time > market.start_time + market.duration {
        return Err(ErrorCode::MarketNotActive.into());
    }

    if ctx.accounts.user.lamports() < amount {
        return Err(ErrorCode::InsufficientUserFunds.into());
    }

    let shares = calculate_shares(amount);

    if choice {
        market.total_yes_shares += shares;
        user_position.yes_shares += shares;
    } else {
        market.total_no_shares += shares;
        user_position.no_shares += shares;
    }

    market.total_funds += amount;

    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.market.to_account_info(),
        },
    );
    anchor_lang::system_program::transfer(cpi_context, amount)?;

    Ok(())
}
