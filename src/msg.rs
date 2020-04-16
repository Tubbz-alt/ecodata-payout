use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm::types::{HumanAddr};
use crate::state::State;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub region: String,
    pub beneficiary: HumanAddr,
    pub oracle: HumanAddr,
    pub ecostate: i64,
    pub total_tokens: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateEcostate {ecostate: i64},
    Lock {},
    Unlock {},
    ChangeBeneficiary {beneficiary: HumanAddr},
    TransferOwnership {owner: HumanAddr},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetState {},
    GetEcostate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub state: State,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EcostateResponse {
    pub ecostate: i64,
}
