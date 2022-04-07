#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Reply, Response, StdError, SubMsg,
    SubMsgResult, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{EchoDataMsg, ExecuteMsg, InstantiateMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-reply-data";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ID: u64 = 0xfa61;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != REPLY_ID {
        return Err(StdError::generic_err("Unknown reply ID").into());
    }

    if let SubMsgResult::Ok(res) = reply.result {
        let data = res.data.unwrap();
        return Ok(Response::new()
            .set_data(data.clone())
            .add_attribute("reply_data", data.to_base64()));
    }

    Err(StdError::generic_err("Invalid result").into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Echo { text } => execute_echo(text),
        ExecuteMsg::SubCall { contract, text } => execute_sub_call(deps, contract, text),
    }
}

pub fn execute_echo(text: String) -> Result<Response, ContractError> {
    let msg = EchoDataMsg { text: text.clone() };
    let data = to_binary(&msg)?;

    Ok(Response::new()
        .set_data(data.clone())
        .add_attribute("method", "echo")
        .add_attribute("text", text)
        .add_attribute("echo_data", data.to_base64()))
}

pub fn execute_sub_call(
    deps: DepsMut,
    contract: String,
    text: String,
) -> Result<Response, ContractError> {
    let contract_addr = deps.api.addr_validate(&contract.to_lowercase())?;
    let msg = ExecuteMsg::Echo { text: text.clone() };

    let contract_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: to_binary(&msg)?,
        funds: vec![],
    }
    .into();

    let sub_msg = SubMsg::reply_on_success(contract_msg, REPLY_ID);

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("method", "sub_call")
        .add_attribute("text", text))
}
