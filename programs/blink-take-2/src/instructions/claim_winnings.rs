use anchor_lang::prelude::*;

use crate::constants::{USER_POSITION_PDA_SEED, MARKET_PDA_SEED, MARKET_CREATION_AUTHORITY};
use crate::errors::ErrorCode;
use crate::state::{Market, UserPosition};

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
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
        seeds = [
          USER_POSITION_PDA_SEED.as_bytes(), 
          market.key().as_ref(), 
          user.key().as_ref()
        ],
        bump,
        has_one = market,
        has_one = user,
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;

    if !market.resolved {
        return Err(ErrorCode::MarketNotResolved.into());
    }

    if user_position.claimed {
        return Err(ErrorCode::AlreadyClaimed.into());
    }

    let winning_outcome = market.winning_outcome.ok_or(ErrorCode::MarketNotResolved)?;

    let winning_shares = if winning_outcome {
        user_position.yes_shares
    } else {
        user_position.no_shares
    };

    let total_winning_shares = if winning_outcome {
        market.total_yes_shares
    } else {
        market.total_no_shares
    };

    let total_funds = market.total_funds as u128;
    let winnings_pool = (total_funds * 95) / 100;
    let team_fee = total_funds - winnings_pool;

    let user_share = (winning_shares as u128 * winnings_pool / total_winning_shares as u128) as u64;

    if market.total_funds < user_share + team_fee as u64 {
        return Err(ErrorCode::InsufficientMarketFunds.into());
    }

    **market.to_account_info().try_borrow_mut_lamports()? -= user_share;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += user_share;

    user_position.claimed = true;
    user_position.yes_shares = 0;
    user_position.no_shares = 0;

    market.total_funds -= user_share;

    Ok(())
}
