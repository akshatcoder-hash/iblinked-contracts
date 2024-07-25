use anchor_lang::prelude::*;

use crate::constants::USER_POSITION_PDA_SEED;
use crate::state::{Market, UserPosition};

#[derive(Accounts)]
pub struct CreateUser<'info> {
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<UserPosition>(),
        seeds = [
          USER_POSITION_PDA_SEED.as_bytes(), 
          market.key().as_ref(), 
          user.key().as_ref()
        ],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_user(ctx: Context<CreateUser>) -> Result<()> {
    let user_position = &mut ctx.accounts.user_position;
    let market = &ctx.accounts.market;
    let user = &ctx.accounts.user;

    user_position.market = market.key();
    user_position.user = user.key();
    user_position.yes_shares = 0;
    user_position.no_shares = 0;
    user_position.claimed = false;

    Ok(())
}
