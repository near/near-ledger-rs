use std::{convert::TryInto};

use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn main() -> Result<(), NEARLedgerError> {
    // TODO #F: replace with actual signature from test after test passes and stops
    // resulting in `TxParsingFail = 0xB005`
    let result_signature_from_speculos_test = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let actions = common::batch_of_all_types_of_actions_v2()
        .into_iter()
        .map(|action| action.try_into().unwrap())
        .collect::<Vec<_>>();
    
    common::get_key_sign_nep_366_and_verify_flow_with_cli_parse(actions, result_signature_from_speculos_test)
}

