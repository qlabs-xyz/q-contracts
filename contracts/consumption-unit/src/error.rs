use cosmwasm_std::StdError;
use cw721::error::Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Cw721ContractError(#[from] Cw721ContractError),
    #[error("WrongInput")]
    WrongInput {},
}
