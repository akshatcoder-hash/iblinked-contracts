use solana_program::pubkey::Pubkey;

pub static TEAM_WALLET: Pubkey =
    solana_program::pubkey!("GerW59qscGWPJarbe8Px3sUVEXJ269Z9RQndYc9MWxCe");
pub static MARKET_CREATION_AUTHORITY: Pubkey =
    solana_program::pubkey!("3xPuKYdk1yBQBtb9g1y1HZQt2udSa8aJLLVKm6mtKaVs");

pub const MARKET_CREATION_FEE: u64 = 100_000_000;
pub const MIN_BET_AMOUNT: u64 = 1_000_000;

pub const MARKET_PDA_SEED: &str = "market";
pub const PRICE_FEED_PDA_SEED: &str = "price_feed";
pub const USER_POSITION_PDA_SEED: &str = "user_position";
