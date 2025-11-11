use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::entry_point;

// Instantiate entrypoint
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, crate::error::ContractError> {
    crate::execute::instantiate(deps, info, msg)
}

// Execute entrypoint
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, crate::error::ContractError> {
    crate::execute::execute(deps, env, info, msg)
}

// Query entrypoint
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    crate::query::query(deps, env, msg)
}

// Migrate entrypoint
#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, crate::error::ContractError> {
    crate::execute::migrate(deps, _msg)
}

