use cosmwasm::errors::{contract_err, unauthorized, Result};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{Env, HumanAddr, Response};

use cw_storage::serialize;

use crate::msg::{EcostateResponse, HandleMsg, InitMsg, QueryMsg, StateResponse};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: InitMsg,
) -> Result<Response> {
    let state = State {
        beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
        ecostate: valid_ecostate(&msg.ecostate)?,
        oracle: deps.api.canonical_address(&msg.oracle)?,
        region: msg.region,
        total_tokens: msg.total_tokens,
        released_tokens: 0,
        owner: env.message.signer,
        is_locked: false,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: HandleMsg,
) -> Result<Response> {
    match msg {
        HandleMsg::Lock {} => try_set_lock(deps, env, true),
        HandleMsg::Unlock {} => try_set_lock(deps, env, false),
        HandleMsg::ChangeBeneficiary { beneficiary } => {
            try_change_beneficiary(deps, env, beneficiary)
        }
        HandleMsg::UpdateEcostate { ecostate } => try_update_ecostate(deps, env, ecostate),
        HandleMsg::TransferOwnership { owner } => try_transfer_ownership(deps, env, owner),
    }
}

fn try_set_lock<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    locked: bool,
) -> Result<Response> {
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            unauthorized()
        } else {
            state.is_locked = locked;
            Ok(state)
        }
    })?;

    Ok(Response::default())
}

fn try_change_beneficiary<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    beneficiary: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        check_lock(&state)?;
        if env.message.signer != state.owner {
            unauthorized()
        } else {
            state.beneficiary = api.canonical_address(&beneficiary)?;
            Ok(state)
        }
    })?;

    Ok(Response::default())
}

fn try_update_ecostate<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    ecostate: i64,
) -> Result<Response> {
    let mut state = config(&mut deps.storage).load()?;
    check_lock(&state)?;

    if env.message.signer != state.oracle {
        unauthorized()?;
    } else {
        valid_ecostate(&ecostate)?;

        let ecostate_delta = ecostate - state.ecostate;
        state.ecostate = ecostate;

        if ecostate_delta > 0 {
            state = execute_payout(state, ecostate_delta)?;
        }

        config(&mut deps.storage).save(&state)?;
    }

    Ok(Response::default())
}

fn execute_payout(mut state: State, ecostate_delta: i64) -> Result<State> {
    let payout_amount = ecostate_delta;

    if payout_amount < 0 {
        contract_err("Error: cannot payout negative ammount")?;
    }

    if state.total_tokens >= payout_amount {
        state.total_tokens -= payout_amount;
        state.released_tokens += payout_amount;
    } else {
        state.released_tokens += state.total_tokens;
        state.total_tokens = 0;
    }

    Ok(state)
}

fn try_transfer_ownership<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    owner: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        check_lock(&state)?;
        if env.message.signer != state.owner {
            unauthorized()
        } else {
            state.owner = api.canonical_address(&owner)?;
            Ok(state)
        }
    })?;

    Ok(Response::default())
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::GetState {} => query_state(deps),
        QueryMsg::GetEcostate {} => query_ecostate(deps),
    }
}

fn query_state<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let state = config_read(&deps.storage).load()?;

    let resp = StateResponse { state };
    serialize(&resp)
}

fn query_ecostate<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let state = config_read(&deps.storage).load()?;

    let resp = EcostateResponse {
        ecostate: state.ecostate,
    };
    serialize(&resp)
}

fn valid_ecostate(ecostate: &i64) -> Result<i64> {
    if *ecostate >= 0 && *ecostate < 10000 {
        Ok(*ecostate)
    } else {
        contract_err("Invalid ecostate. Value must be integer between 0 and 10000")
    }
}

fn check_lock(state: &State) -> Result<()> {
    if state.is_locked {
        contract_err("Contract is locked.")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::errors::Error;
    use cosmwasm::mock::{dependencies, mock_env};
    use cosmwasm::types::coin;

    use cw_storage::deserialize;

    fn init_helper<S: Storage, A: Api>(deps: &mut Extern<S, A>) -> Result<Response> {
        let msg = InitMsg {
            beneficiary: HumanAddr::from("beneficiary"),
            ecostate: 3500,
            oracle: HumanAddr::from("oracle"),
            region: String::from("angeles national forest"),
            total_tokens: 100000,
        };

        let env = mock_env(&deps.api, "creator", &coin("1000", "earth"), &[]);

        // we can just call .unwrap() to assert this was a success
        init(deps, env, msg)
    }

    #[test]
    fn proper_initialization() {
        let mut deps = dependencies(20);
        let res = init_helper(&mut deps).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetEcostate {}).unwrap();
        let value: EcostateResponse = deserialize(&res).unwrap();
        assert_eq!(3500, value.ecostate);
    }

    #[test]
    fn ecostate_update_with_payout() {
        let mut deps = dependencies(20);
        let _res = init_helper(&mut deps).unwrap();

        // oracle can update ecostate
        let env = mock_env(&deps.api, "oracle", &coin("2", "token"), &[]);
        let msg = HandleMsg::UpdateEcostate { ecostate: 5000 };
        let _res = handle(&mut deps, env, msg).unwrap();

        // ecostate should have updated successfully
        let res = query(&deps, QueryMsg::GetEcostate {}).unwrap();
        let value: EcostateResponse = deserialize(&res).unwrap();
        assert_eq!(5000, value.ecostate);

        // non-oracle account cannot update ecostate
        let env = mock_env(&deps.api, "anyone", &coin("2", "token"), &[]);
        let msg = HandleMsg::UpdateEcostate { ecostate: 5000 };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(Error::Unauthorized { .. }) => {}
            _ => panic!("Ecostate should not be updatable by non-oracle account"),
        }

        // payout should have completed successfully
        let res = query(&deps, QueryMsg::GetState {}).unwrap();
        let value: StateResponse = deserialize(&res).unwrap();
        assert_eq!(5000, value.state.ecostate);
        assert_eq!(98500, value.state.total_tokens);
        assert_eq!(1500, value.state.released_tokens);
    }

    #[test]
    fn ecostate_update_no_payout() {
        let mut deps = dependencies(20);
        let _res = init_helper(&mut deps).unwrap();

        // oracle can update ecostate
        let env = mock_env(&deps.api, "oracle", &coin("2", "token"), &[]);
        let msg = HandleMsg::UpdateEcostate { ecostate: 3000 };
        let _res = handle(&mut deps, env, msg).unwrap();

        // ecostate should have updated successfully, with no payout made
        let res = query(&deps, QueryMsg::GetState {}).unwrap();
        let value: StateResponse = deserialize(&res).unwrap();
        assert_eq!(3000, value.state.ecostate);
        assert_eq!(100000, value.state.total_tokens);
        assert_eq!(0, value.state.released_tokens);
    }
}
