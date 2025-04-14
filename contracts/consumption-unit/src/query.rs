use crate::state::{CUConfig, CU_CONFIG};
use crate::types::ConsumptionUnitData;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CUConfig)]
    GetConfig {},

    // TODO add Cw721 config as well
    #[returns(cw721::msg::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    #[returns(cw721::msg::NumTokensResponse)]
    NumTokens {},

    #[returns(cw_ownable::Ownership<String>)]
    GetMinterOwnership {},

    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},

    #[returns(cw721::msg::NftInfoResponse<ConsumptionUnitData>)]
    NftInfo { token_id: String },

    /// Returns all tokens owned by the given address.
    /// Same as `AllTokens` but with owner filter.
    #[returns(cw721::msg::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(cw721::msg::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_get_config(deps)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_json_binary(&cw721::query::query_owner_of(
            deps,
            &env,
            token_id,
            include_expired.is_some(),
        )?),
        QueryMsg::NumTokens {} => to_json_binary(&cw721::query::query_num_tokens(deps.storage)?),
        QueryMsg::GetMinterOwnership {} => {
            to_json_binary(&cw721::query::query_minter_ownership(deps.storage)?)
        }
        QueryMsg::GetCreatorOwnership {} => {
            to_json_binary(&cw721::query::query_creator_ownership(deps.storage)?)
        }
        QueryMsg::NftInfo { token_id } => to_json_binary(&cw721::query::query_nft_info::<
            ConsumptionUnitData,
        >(deps.storage, token_id)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_json_binary(&cw721::query::query_tokens(
            deps,
            &env,
            owner,
            start_after,
            limit,
        )?),
        QueryMsg::AllTokens { start_after, limit } => to_json_binary(
            &cw721::query::query_all_tokens(deps, &env, start_after, limit)?,
        ),
    }
}

// Query

fn query_get_config(deps: Deps) -> StdResult<CUConfig> {
    CU_CONFIG.load(deps.storage)
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::{ConsumptionUnitCollectionExtension, InstantiateMsg};
    use crate::query::{query, QueryMsg};
    use cosmwasm_std::Addr;
    use cw20::Denom;
    use cw721::msg::NumTokensResponse;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use cw_ownable::Ownership;

    #[test]
    fn test_query_config() {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg {
            name: "consumption unit".to_string(),
            symbol: "cu".to_string(),
            collection_info_extension: ConsumptionUnitCollectionExtension {
                settlement_token: Denom::Cw20(Addr::unchecked("settlement")),
                native_token: Denom::Native("native".to_string()),
                price_oracle: Addr::unchecked("price_oracle"),
            },
            minter: None,
            creator: None,
            withdraw_address: None,
        };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "cu1", None)
            .unwrap();

        let response: NumTokensResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::NumTokens {})
            .unwrap();
        assert_eq!(response.count, 0);

        let response: Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetMinterOwnership {})
            .unwrap();

        assert_eq!(response.owner.unwrap(), owner.to_string());

        let response: Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetCreatorOwnership {})
            .unwrap();

        assert_eq!(response.owner.unwrap(), owner.to_string());
    }
}
