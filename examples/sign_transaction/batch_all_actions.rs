use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);
    tx.actions = common::batch_of_all_types_of_actions_v1(ledger_pub_key);
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    // TODO #B0: run it with live device
    let result_signature_from_speculos_test = hex::decode("bd5420d0279f398893231b505b004403c682c8ef8e2b5181d0b007dfbc802dacfadbd20883938a236ccd78f388d2b52b522574d2a3c682c380c814cbf6ccad02").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
