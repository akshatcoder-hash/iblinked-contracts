use solana_program::pubkey::Pubkey;

pub static TEAM_WALLET: Pubkey =
    solana_program::pubkey!("GerW59qscGWPJarbe8Px3sUVEXJ269Z9RQndYc9MWxCe");
pub static MARKET_CREATION_AUTHORITY: Pubkey =
    solana_program::pubkey!("4EUxX4o9FHcspLkMnMrarfJ2fWkjaJvYwLntHusxYEQN");

pub const MARKET_CREATION_FEE: u64 = 100_000_000;
pub const MIN_BET_AMOUNT: u64 = 1_000_000;
pub const STALENESS_THRESHOLD: u64 = 60 * 5;

pub const MARKET_PDA_SEED: &str = "market";
pub const PRICE_FEED_CONFIG_PDA_SEED: &str = "price_feed_config";
pub const USER_POSITION_PDA_SEED: &str = "user_position";
