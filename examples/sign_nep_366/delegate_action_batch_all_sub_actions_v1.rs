use std::{convert::TryInto};

use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn main() -> Result<(), NEARLedgerError> {
    let result_signature_from_speculos_test = hex::decode("3f671b1d2ba42132e78c39dad35848ef0ee67858d85bd63cb1ce9e03d629c74cec9529add083aa0de6dbd45d372baa67bd8f8e49e76297a87cec1dc7084ae80d").unwrap();
    let actions = common::batch_of_all_types_of_actions_v1()
        .into_iter()
        .map(|action| action.try_into().unwrap())
        .collect::<Vec<_>>();
    
    common::get_key_sign_nep_366_and_verify_flow_with_cli_parse(actions, result_signature_from_speculos_test)
}

