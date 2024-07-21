use anchor_lang::prelude::*;

use crate::constants::TEAM_WALLET;
use crate::errors::ErrorCode;
use crate::state::Market;

#[derive(Accounts)]
pub struct WithdrawTeamFee<'info> {
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
    pub authority: Signer<'info>,
    #[account(mut, address = TEAM_WALLET)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub team_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_team_fee(ctx: Context<WithdrawTeamFee>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let current_time = Clock::get()?.unix_timestamp;

    if current_time < market.team_fee_unlock_time {
        return Err(ErrorCode::TeamFeeTimelockNotExpired.into());
    }

    let team_fee = (market.total_funds as u128 * 5) / 100;

    **market.to_account_info().try_borrow_mut_lamports()? -= team_fee as u64;
    **ctx
        .accounts
        .team_wallet
        .to_account_info()
        .try_borrow_mut_lamports()? += team_fee as u64;

    market.team_fee_paid = true;
    market.total_funds -= team_fee as u64;

    Ok(())
}
