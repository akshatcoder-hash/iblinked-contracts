use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
use pyth_sdk_solana::state::{AccountType, PriceAccount, PriceStatus};
use std::mem::size_of;
use crate::state::{Market, UserPosition};
use crate::errors::ErrorCode;
use crate::utils::{calculate_shares, calculate_refund_amount};

pub static TEAM_WALLET: Pubkey = solana_program::pubkey!("GerW59qscGWPJarbe8Px3sUVEXJ269Z9RQndYc9MWxCe");
pub const MARKET_CREATION_FEE: u64 = 100_000_000; // 0.1 SOL

#[derive(Accounts)]
#[instruction(memecoin_symbol: String)]
pub struct CreateMarket<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [b"market", authority.key().as_ref(), memecoin_symbol.as_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub price_feed: Account<'info, MockPythFeed>,
    #[account(mut, address = TEAM_WALLET)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub team_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserPosition>(),
        seeds = [b"user_position", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
    pub authority: Signer<'info>,
    /// CHECK: This account is checked in the instruction
    pub price_feed: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"user_position", market.key().as_ref(), user.key().as_ref()],
        bump,
        has_one = market,
        has_one = user,
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, address = TEAM_WALLET)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub team_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

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

#[derive(Accounts)]
pub struct CancelBet<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"user_position", market.key().as_ref(), user.key().as_ref()],
        bump,
        has_one = market,
        has_one = user,
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_market(
    ctx: Context<CreateMarket>,
    memecoin_symbol: String,
    feed_id: String,
    duration: u64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.memecoin_symbol = memecoin_symbol;
    market.feed_id = feed_id;
    market.duration = duration;
    market.start_time = Clock::get()?.unix_timestamp as u64;
    market.total_yes_shares = 0;
    market.total_no_shares = 0;
    market.resolved = false;
    market.winning_outcome = None;
    market.total_funds = 0;
    market.authority = ctx.accounts.authority.key();
    
    // Fetch and set initial price from our simplified mock
    let mock_price_feed = &ctx.accounts.price_feed;
    market.initial_price = Some(mock_price_feed.price);

    // Set team fee unlock time (7 days after market resolution)
    market.team_fee_unlock_time = (market.start_time + market.duration + 7 * 24 * 60 * 60) as i64;

    // Transfer market creation fee
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

pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, choice: bool) -> Result<()> {
    const MIN_BET_AMOUNT: u64 = 1_000_000; 

    if amount < MIN_BET_AMOUNT {
        return Err(ErrorCode::BetAmountTooLow.into());
    }

    let market = &mut ctx.accounts.market;
    
    // Ensure the market is still active
    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time < market.start_time || current_time > market.start_time + market.duration {
        return Err(ErrorCode::MarketNotActive.into());
    }

    // Check if user has sufficient funds
    if ctx.accounts.user.lamports() < amount {
        return Err(ErrorCode::InsufficientUserFunds.into());
    }

    let user_position = &mut ctx.accounts.user_position;
    let shares = calculate_shares(amount);
    
    if choice {
        market.total_yes_shares += shares;
        user_position.yes_shares += shares;
    } else {
        market.total_no_shares += shares;
        user_position.no_shares += shares;
    }

    market.total_funds += amount;

    // Transfer funds from user to market PDA
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

pub fn resolve_market(ctx: Context<ResolveMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Ensure market duration has passed
    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time <= market.start_time + market.duration {
        return Err(ErrorCode::MarketNotExpired.into());
    }

    if market.resolved {
        return Err(ErrorCode::MarketAlreadyResolved.into());
    }

    let price_feed: PriceFeed = load_price_feed_from_account_info(&ctx.accounts.price_feed)
        .map_err(|_| ErrorCode::PriceFetchFailed)?;
    let current_price = price_feed.get_price_unchecked();

    let final_price = current_price.price;
    let initial_price = market.initial_price.ok_or(ErrorCode::InitialPriceNotSet)?;

    market.resolved = true;
    market.winning_outcome = Some(final_price > initial_price);
    market.final_price = Some(final_price);

    Ok(())
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

    // Ensure the market has enough funds
    if market.total_funds < user_share + team_fee as u64 {
        return Err(ErrorCode::InsufficientMarketFunds.into());
    }

    // Transfer winnings from market PDA to user
    **market.to_account_info().try_borrow_mut_lamports()? -= user_share;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += user_share;

    // Mark user position as claimed and reset shares
    user_position.claimed = true;
    user_position.yes_shares = 0;
    user_position.no_shares = 0;

    // Update market's total funds
    market.total_funds -= user_share;

    Ok(())
}

pub fn withdraw_team_fee(ctx: Context<WithdrawTeamFee>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let current_time = Clock::get()?.unix_timestamp;

    if current_time < market.team_fee_unlock_time {
        return Err(ErrorCode::TeamFeeTimelockNotExpired.into());
    }

    let team_fee = (market.total_funds as u128 * 5) / 100;

    // Transfer team fee from market PDA to team account
    **market.to_account_info().try_borrow_mut_lamports()? -= team_fee as u64;
    **ctx.accounts.team_wallet.to_account_info().try_borrow_mut_lamports()? += team_fee as u64;

    market.team_fee_paid = true;
    market.total_funds -= team_fee as u64;

    Ok(())
}

pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let user_position = &mut ctx.accounts.user_position;

    // Ensure the market hasn't started yet
    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time >= market.start_time {
        return Err(ErrorCode::MarketAlreadyStarted.into());
    }

    let total_shares = user_position.yes_shares + user_position.no_shares;
    let refund_amount = calculate_refund_amount(total_shares, 0, market.duration);

    // Update market state
    market.total_yes_shares -= user_position.yes_shares;
    market.total_no_shares -= user_position.no_shares;
    market.total_funds -= refund_amount;

    // Refund user
    **market.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += refund_amount;

    // Reset user position
    user_position.yes_shares = 0;
    user_position.no_shares = 0;

    Ok(())
}

#[derive(Accounts)]
#[instruction(price: i64)]
pub struct InitializeMockPythFeed<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 8, // discriminator + i64
        seeds = [b"mock_pyth_feed", payer.key().as_ref()],
        bump
    )]
    pub price_feed: Account<'info, MockPythFeed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct MockPythFeed {
    pub price: i64,
}

pub fn initialize_mock_pyth_feed(ctx: Context<InitializeMockPythFeed>, price: i64) -> Result<()> {
    ctx.accounts.price_feed.price = price;
    Ok(())
}
