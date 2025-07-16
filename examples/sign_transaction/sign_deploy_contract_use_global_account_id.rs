use std::str::FromStr;

use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);

    let referenced_account_id =
        near_primitives::types::AccountId::from_str("simple-package-verify-rs-ci.testnet").unwrap();
    log::warn!("referenced_account_id: {}", referenced_account_id);
    let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
        near_primitives::action::UseGlobalContractAction {
            contract_identifier: near_primitives::action::GlobalContractIdentifier::AccountId(
                referenced_account_id,
            ),
        },
    ));
    log::warn!(
        "action bytes: {:x?}",
        borsh::to_vec(&action).expect("no ser err")
    );
    tx.actions = vec![action];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    // TODO #F: replace with actual signature from test after test passes and stops
    // resulting in `TxParsingFail = 0xB005`
    let result_signature_from_speculos_test = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
