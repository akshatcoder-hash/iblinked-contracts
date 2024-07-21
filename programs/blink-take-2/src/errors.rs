use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Market is not active")]
    MarketNotActive,
    #[msg("Insufficient Funds")]
    InsufficientFunds,
    #[msg("The market is already resolved.")]
    MarketAlreadyResolved,
    #[msg("The market is not resolved yet.")]
    MarketNotResolved,
    #[msg("Failed to fetch price from Pyth")]
    PriceFetchFailed,
    #[msg("Invalid market duration")]
    InvalidDuration,
    #[msg("Market has not expired yet")]
    MarketNotExpired,
    #[msg("Not a winner")]
    NotAWinner,
    #[msg("Initial Price not set")]
    InitialPriceNotSet,
    #[msg("Winnings already claimed")]
    AlreadyClaimed,
    #[msg("Insufficient funds in the market")]
    InsufficientMarketFunds,
    #[msg("Bet amount is too low")]
    BetAmountTooLow,
    #[msg("Insufficient user funds")]
    InsufficientUserFunds,
    #[msg("Market has already started")]
    MarketAlreadyStarted,
    #[msg("Team fee timelock has not expired")]
    TeamFeeTimelockNotExpired,
    #[msg("Internal Pyth error")]
    PythError,
    #[msg("Program should not try to serialize a price account")]
    TryToSerializePriceAccount,
}
