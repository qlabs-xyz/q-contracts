use crate::error::ContractError;
use crate::msg::{ConsumptionUnitExtensionUpdate, ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{CUConfig, CU_CONFIG};
use crate::types::{ConsumptionUnitData, ConsumptionUnitNft, ConsumptionUnitState};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use cw721::error::Cw721ContractError;
use cw721::execute::{assert_minter, check_can_send};
use cw721::state::{CollectionInfo, Cw721Config};
use cw721::OwnershipError;

const CONTRACT_NAME: &str = "gemlabs.io:consumption-unit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // TODO save ConsumptionUnitCollectionExtension instead of CUConfig?
    let cfg = CUConfig {
        settlement_token: msg.collection_info_extension.settlement_token.clone(),
        native_token: msg.collection_info_extension.native_token.clone(),
        price_oracle: msg.collection_info_extension.price_oracle.clone(),
    };

    CU_CONFIG.save(deps.storage, &cfg)?;

    // TODO reuse logic from CW721 fully?

    // ---- update collection info before(!) creator and minter is set ----
    let collection_metadata_msg = CollectionInfo {
        name: msg.name,
        symbol: msg.symbol,
        updated_at: env.block.time,
    };

    let config = Cw721Config::<ConsumptionUnitData>::default();
    config
        .collection_info
        .save(deps.storage, &collection_metadata_msg)?;

    // ---- set minter and creator ----
    // use info.sender if None is passed
    let minter: &str = match msg.minter.as_deref() {
        Some(minter) => minter,
        None => info.sender.as_str(),
    };
    cw721::execute::initialize_minter(deps.storage, deps.api, Some(minter))?;

    // use info.sender if None is passed
    let creator: &str = match msg.creator.as_deref() {
        Some(creator) => creator,
        None => info.sender.as_str(),
    };
    cw721::execute::initialize_creator(deps.storage, deps.api, Some(creator))?;

    if msg.withdraw_address.clone().is_some() {
        return Err(ContractError::WrongInput {});
    }

    Ok(Response::default()
        .add_attribute("action", "consumption-unit::instantiate")
        .add_event(
            Event::new("consumption-unit::instantiate")
                .add_attribute("minter", minter)
                .add_attribute("creator", creator),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => execute_mint(deps, &env, &info, token_id, owner, token_uri, extension),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, &env, &info, token_id),
        ExecuteMsg::UpdateNftInfo {
            token_id,
            extension,
        } => execute_update_nft_info(deps, &env, &info, token_id, extension),
    }
}

fn execute_update_nft_info(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: String,
    update: ConsumptionUnitExtensionUpdate,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<ConsumptionUnitData>::default();

    match update {
        ConsumptionUnitExtensionUpdate::UpdatePool {
            new_commitment_tier_id,
        } => {
            let mut current_nft_info = config.nft_info.load(deps.storage, &token_id)?;
            if current_nft_info.owner != info.sender {
                return Err(ContractError::Cw721ContractError(
                    cw721::error::Cw721ContractError::Ownership(OwnershipError::NotOwner),
                ));
            }

            if current_nft_info.extension.state == ConsumptionUnitState::Selected {
                return Err(ContractError::WrongInput {});
            }

            current_nft_info.extension = current_nft_info
                .extension
                .update_tier(new_commitment_tier_id, env);

            config
                .nft_info
                .save(deps.storage, &token_id, &current_nft_info)?;

            Ok(Response::new()
                .add_attribute("action", "consumption-unit::update_nft_info")
                .add_event(
                    Event::new("consumption-unit::update_nft_info")
                        .add_attribute("token_id", token_id)
                        .add_attribute(
                            "new_commitment_pool_id",
                            new_commitment_tier_id.to_string(),
                        ),
                ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_mint(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
    owner: String,
    token_uri: Option<String>,
    extension: ConsumptionUnitData,
) -> Result<Response, ContractError> {
    assert_minter(deps.storage, &info.sender)?;
    // validate owner
    let owner_addr = deps.api.addr_validate(&owner)?;

    let config = Cw721Config::<ConsumptionUnitData>::default();

    // create the token

    let token = ConsumptionUnitNft {
        owner: owner_addr,
        approvals: vec![],
        token_uri,
        extension,
    };

    config
        .nft_info
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(Cw721ContractError::Claimed {}),
            None => Ok(token),
        })?;

    config.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "consumption-unit::mint")
        .add_event(
            Event::new("consumption-unit::mint")
                .add_attribute("token_id", token_id)
                .add_attribute("owner", owner),
        ))
}

fn execute_burn(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<ConsumptionUnitData>::default();
    let token = config.nft_info.load(deps.storage, &token_id)?;
    check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.nft_info.remove(deps.storage, &token_id)?;
    config.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "consumption-unit::burn")
        .add_event(
            Event::new("consumption-unit::burn")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("token_id", token_id),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}
