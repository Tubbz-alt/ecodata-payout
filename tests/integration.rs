//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20);
//!    to
//!      let mut deps = mock_instance(WASM);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
//! 5. When matching on error codes, you can not use Error types, but rather must use strings:
//!      match res {
//!          Err(Error::Unauthorized{..}) => {},
//!          _ => panic!("Must return unauthorized error"),
//!      }
//!    becomes:
//!      match res {
//!         ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
//!         _ => panic!("Expected error"),
//!      }

use cosmwasm::mock::{mock_env, MockStorage, MockApi};
use cosmwasm::types::{coin, ContractResult, HumanAddr};

use cosmwasm_vm::Instance;
use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use cw_storage::deserialize;

use ecodata_payout::msg::{EcostateResponse, HandleMsg, InitMsg, QueryMsg, StateResponse};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/ecodata_payout.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");


fn init_helper(deps: &mut Instance<MockStorage, MockApi>) -> ContractResult {
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
    let mut deps = mock_instance(WASM);
    let res = init_helper(&mut deps).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::GetEcostate {}).unwrap();
    let value: EcostateResponse = deserialize(res.as_slice()).unwrap();
    assert_eq!(3500, value.ecostate);
}

#[test]
fn ecostate_update_with_payout() {
    let mut deps = mock_instance(WASM);
    let _res = init_helper(&mut deps).unwrap();

    // oracle can update ecostate
    let env = mock_env(&deps.api, "oracle", &coin("2", "token"), &[]);
    let msg = HandleMsg::UpdateEcostate { ecostate: 5000 };
    let _res = handle(&mut deps, env, msg).unwrap();

    // ecostate should have updated successfully
    let res = query(&mut deps, QueryMsg::GetEcostate {}).unwrap();
    let value: EcostateResponse = deserialize(res.as_slice()).unwrap();
    assert_eq!(5000, value.ecostate);

    // non-oracle account cannot update ecostate
    let env = mock_env(&deps.api, "anyone", &coin("2", "token"), &[]);
    let msg = HandleMsg::UpdateEcostate { ecostate: 5000 };
    let res = handle(&mut deps, env, msg);
    match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Ecostate should not  be updatable by non-oracle"),
    }

    // payout should have completed successfully
    let res = query(&mut deps, QueryMsg::GetState {}).unwrap();
    let value: StateResponse = deserialize(res.as_slice()).unwrap();
    assert_eq!(5000, value.state.ecostate);
    assert_eq!(98500, value.state.total_tokens);
    assert_eq!(1500, value.state.released_tokens);
}

#[test]
fn ecostate_update_no_payout() {
    let mut deps = mock_instance(WASM);
    let _res = init_helper(&mut deps).unwrap();

    // oracle can update ecostate
    let env = mock_env(&deps.api, "oracle", &coin("2", "token"), &[]);
    let msg = HandleMsg::UpdateEcostate { ecostate: 3000 };
    let _res = handle(&mut deps, env, msg).unwrap();

    // ecostate should have updated successfully, with no payout made
    let res = query(&mut deps, QueryMsg::GetState {}).unwrap();
    let value: StateResponse = deserialize(res.as_slice()).unwrap();
    assert_eq!(3000, value.state.ecostate);
    assert_eq!(100000, value.state.total_tokens);
    assert_eq!(0, value.state.released_tokens);
}
