use cosmwasm_std::{Addr, Deps, Empty, Env, Order, StdResult, Storage};
use cw_ownable::Ownership;
use cw_storage_plus::Bound;

use crate::msg::ContractInfoResponse;
use crate::traits::Cw721CollectionConfig;
use crate::{
    msg::{NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse},
    state::{Cw721Config, CREATOR, MINTER},
    traits::Cw721State,
};

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;

// --- query helpers ---
pub fn query_minter_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    MINTER.get_ownership(storage)
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

pub fn query_num_tokens(storage: &dyn Storage) -> StdResult<NumTokensResponse> {
    let count = Cw721Config::<Option<Empty>, Option<Empty>>::default().token_count(storage)?;
    Ok(NumTokensResponse { count })
}

pub fn query_nft_info<TNftExtension>(
    storage: &dyn Storage,
    token_id: String,
) -> StdResult<NftInfoResponse<TNftExtension>>
where
    TNftExtension: Cw721State,
{
    let info = Cw721Config::<TNftExtension, Option<Empty>>::default()
        .nft_info
        .load(storage, &token_id)?;
    Ok(NftInfoResponse {
        extension: info.extension,
    })
}

pub fn query_contract_info<TCollectionConfig>(
    storage: &dyn Storage,
) -> StdResult<ContractInfoResponse<TCollectionConfig>>
where
    TCollectionConfig: Cw721CollectionConfig,
{
    let info = Cw721Config::<Option<Empty>, TCollectionConfig>::default()
        .collection_info
        .load(storage)?;
    let config = Cw721Config::<Option<Empty>, TCollectionConfig>::default()
        .collection_config
        .load(storage)?;

    Ok(ContractInfoResponse {
        collection_info: info,
        collection_config: config,
    })
}

pub fn query_owner_of(
    storage: &dyn Storage,
    _env: &Env,
    token_id: String,
) -> StdResult<OwnerOfResponse> {
    let nft_info = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .load(storage, &token_id)?;
    Ok(OwnerOfResponse {
        owner: nft_info.owner.to_string(),
    })
}

pub fn query_tokens(
    deps: Deps,
    _env: &Env,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let owner_addr = deps.api.addr_validate(&owner)?;
    let tokens: Vec<String> = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(TokensResponse { tokens })
}

pub fn query_all_tokens(
    deps: Deps,
    _env: &Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let tokens: StdResult<Vec<String>> = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| k))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}
