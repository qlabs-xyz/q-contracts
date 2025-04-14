use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw20::Denom;
use cw_storage_plus::Item;

#[cw_serde]
pub struct CUConfig {
    pub settlement_token: Denom,
    pub native_token: Denom,
    pub price_oracle: Addr,
}

pub(crate) const CU_CONFIG: Item<CUConfig> = Item::new("cu_config");

// NB: another state is reused from CW721
