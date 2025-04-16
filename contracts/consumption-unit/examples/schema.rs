use consumption_unit::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use consumption_unit::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
