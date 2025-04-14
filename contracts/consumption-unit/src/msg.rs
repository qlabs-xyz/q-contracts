use crate::types::ConsumptionUnitData;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw20::Denom;
use cw721::msg::Cw721InstantiateMsg;

#[cw_serde]
pub struct ConsumptionUnitCollectionExtension {
    pub settlement_token: Denom,
    pub native_token: Denom,
    /// Address of the price Oracle to query floor prices
    pub price_oracle: Addr,
}

pub type InstantiateMsg = Cw721InstantiateMsg<ConsumptionUnitCollectionExtension>;

#[cw_serde]
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: ConsumptionUnitData,
    },

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    /// Extension msg
    UpdateNftInfo {
        token_id: String,
        extension: ConsumptionUnitExtensionUpdate,
    },
}

#[cw_serde]
pub enum ConsumptionUnitExtensionUpdate {
    /// Updates the pool id for the given NFT, can be performed by user only.
    /// When updating the pool a new price will be fetched.
    UpdatePool { new_commitment_tier_id: u16 },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
