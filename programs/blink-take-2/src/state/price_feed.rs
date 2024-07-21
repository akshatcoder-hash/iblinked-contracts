use std::{ops::Deref, str::FromStr};

use anchor_lang::prelude::*;
use pyth_sdk_solana::state::{load_price_account, SolanaPriceAccount};

use crate::errors::ErrorCode;

#[account]
pub struct PriceFeedConfig {
    pub price_feed: Pubkey,
}

#[derive(Clone)]
pub struct PriceFeed(pyth_sdk_solana::PriceFeed);

impl anchor_lang::Owner for PriceFeed {
    fn owner() -> Pubkey {
        // Make sure the owner is the pyth oracle account on solana devnet
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        return Pubkey::from_str(&oracle_addr).unwrap();
    }
}

impl anchor_lang::AccountDeserialize for PriceFeed {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self> {
        let account: &SolanaPriceAccount =
            load_price_account(data).map_err(|_x| error!(ErrorCode::PythError))?;

        let zeros: [u8; 32] = [0; 32];
        let dummy_key = solana_program::pubkey::Pubkey::from(zeros);
        let feed = account.to_price_feed(&dummy_key);
        return Ok(PriceFeed(feed));
    }
}

impl anchor_lang::AccountSerialize for PriceFeed {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> std::result::Result<(), Error> {
        Err(error!(ErrorCode::TryToSerializePriceAccount))
    }
}

impl Deref for PriceFeed {
    type Target = pyth_sdk_solana::PriceFeed;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
