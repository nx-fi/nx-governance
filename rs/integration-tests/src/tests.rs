use candid::{encode_one, Decode, Encode};
use candid::{CandidType, Principal};
use nx_gov_main::*;

use pocket_ic::{PocketIc, WasmResult};
use serde::{Deserialize, Serialize};

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;
use rand::Rng;
fn generate_random_principal() -> Principal {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 29] = rng.gen(); // Generate 29 random bytes
    Principal::from_slice(&random_bytes)
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq, Eq)]
pub struct SimpleValidatorPayload {
    gov_main_principal: Principal,
}

#[test]
fn test_rs_nx_gov_main_src_test_rs() {
    println!("Test: test_rs_nx_gov_main_src_test_rs");
    let pic = PocketIc::new();
    let nx_gov_main_can_id = pic.create_canister();
    pic.add_cycles(nx_gov_main_can_id, INIT_CYCLES);
    let nx_gov_main_wasm = nx_gov_main_wasm();
    pic.install_canister(nx_gov_main_can_id, nx_gov_main_wasm, vec![], None);

    let metadata = ProposalMetadata {
        name: String::from("Test proposal 0"),
        description: String::from("AAA"),
        memo: vec![],
    };
    let payload = ProposalPayload {
        depends_on: vec![],
        messages: vec![],
    };
    let activates = Schedule::At(0);
    let expires = Schedule::In(100000000000000000);
    let auto_execute: bool = false;
    let encoded_arg = Encode!(&metadata, &payload, &activates, &expires, &auto_execute).unwrap();
    let res = pic
        .update_call(
            nx_gov_main_can_id,
            Principal::anonymous(),
            "submit",
            encoded_arg,
        )
        .expect("Failed to call canister");
    match res {
        WasmResult::Reply(bytes) => {
            match Decode!(&bytes, Result<Index, ReturnError>) {
                Ok(res) => {
                    match res {
                        Ok(index) => {
                            println!("Received index: {}", index);
                            assert_eq!(index, 0);
                        }
                        Err(error) => {
                            // Handle the error here
                            eprintln!("An error occurred: {:?}", error);
                        }
                    }
                }
                Err(_) => println!("An error occurred while decoding"),
            }
        }
        WasmResult::Reject(msg) => println!("Reject: {}", msg),
    }

    let skip: u64 = 0;
    let take: u64 = 1;
    let encoded_arg = Encode!(&skip, &take).unwrap();
    let res = pic
        .query_call(
            nx_gov_main_can_id,
            Principal::anonymous(),
            "get_proposal_states",
            encoded_arg,
        )
        .expect("Failed to call canister");
    match res {
        WasmResult::Reply(bytes) => match Decode!(&bytes, Vec<ProposalState>) {
            Ok(res) => {
                for state in &res {
                    println!("States: {:?}", state);
                    assert_eq!(*state, ProposalState::Submitted);
                }
            }
            Err(_) => println!("An error occurred while decoding"),
        },
        WasmResult::Reject(msg) => println!("Reject: {}", msg),
    }

    let simple_validator_can_id = pic.create_canister();
    pic.add_cycles(simple_validator_can_id, INIT_CYCLES);
    let simple_validator_wasm = simple_validator_wasm();
    pic.install_canister(
        simple_validator_can_id,
        simple_validator_wasm,
        encode_one(nx_gov_main_can_id).unwrap(),
        None,
    );

    let multisig_voting_can_id = pic.create_canister();
    pic.add_cycles(multisig_voting_can_id, INIT_CYCLES);
    let multisig_voting_wasm = multisig_voting_wasm();

    let votes_required: u64 = 1;
    let total_votes: u64 = 1;
    let vote_principal = generate_random_principal();
    let encoded_arg = Encode!(
        &nx_gov_main_can_id,
        &votes_required,
        &total_votes,
        &vec![vote_principal]
    )
    .unwrap();
    pic.install_canister(
        multisig_voting_can_id,
        multisig_voting_wasm,
        encoded_arg,
        None,
    );
}

fn nx_gov_main_wasm() -> Vec<u8> {
    let wasm_path = std::env::var_os("NX_GOV_MAIN_WASM").expect("Missing nx-gov-main wasm file");
    if let Ok(regular_string) = wasm_path.clone().into_string() {
        println!("{regular_string}");
    } else {
        println!("Not a string");
    };
    std::fs::read(wasm_path).unwrap()
}

fn simple_validator_wasm() -> Vec<u8> {
    let wasm_path =
        std::env::var_os("SIMPLE_VALIDATOR_WASM").expect("Missing simple-validator wasm file");
    if let Ok(regular_string) = wasm_path.clone().into_string() {
        println!("{regular_string}");
    } else {
        println!("Not a string");
    };
    std::fs::read(wasm_path).unwrap()
}

// Load multisig-voting canister
fn multisig_voting_wasm() -> Vec<u8> {
    let wasm_path =
        std::env::var_os("MULTISIG_VOTING_WASM").expect("Missing multisig-voting wasm file");
    if let Ok(regular_string) = wasm_path.clone().into_string() {
        println!("{regular_string}");
    } else {
        println!("Not a string");
    };
    std::fs::read(wasm_path).unwrap()
}
