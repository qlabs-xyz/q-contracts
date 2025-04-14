use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Env, Timestamp, Uint128};
use cw721::state::NftInfo;

/// ConsumptionUnit public data
#[cw_serde]
pub struct ConsumptionUnitData {
    /// The value of Consumption Unit in Settlement Tokens
    pub consumption_value: Uint128,
    /// Sum of Nominal Qty from Consumption Records
    pub nominal_quantity: Uint128,
    /// Nominal currency from Consumption Records
    pub nominal_currency: String,
    /// Where the CU is allocated by the User.
    /// A user can change commitment Pool at any time prior to CU NFT selection in raffle
    pub commitment_tier: u16,
    /// State of the record
    pub state: ConsumptionUnitState,
    /// Calculated according to initial Native Coin Price, PGT, and allocated Commitment Pool.
    /// FloorPrice is to be re-calculated each time out of the update of the Commitment Pool
    pub floor_price: Decimal,
    /// Hashes identifying consumption records batch
    pub hashes: Vec<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cw_serde]
pub enum ConsumptionUnitState {
    /// Created on the Network
    Reflected,
    /// Participating in Raffle (Commitment pool and consequently floorPrice can be changed)
    Nominated,
    /// Was selected as a winner in Raffle
    /// (Commitment pool and consequently floorPrice cannot be changed)
    Selected,
}

pub type ConsumptionUnitNft = NftInfo<ConsumptionUnitData>;

impl cw721::traits::Cw721State for ConsumptionUnitData {}
impl cw721::traits::Cw721CustomMsg for ConsumptionUnitData {}

impl ConsumptionUnitData {
    pub fn update_tier(mut self, new_tier_id: u16, env: &Env) -> Self {
        self.commitment_tier = new_tier_id;
        self.updated_at = env.block.time;
        self
    }
}
