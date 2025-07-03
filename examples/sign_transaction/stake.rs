use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);
    let public_key = common::get_static_secp256k1_public_key();
    tx.actions = vec![near_primitives::transaction::Action::Stake(Box::new(
        near_primitives::transaction::StakeAction {
            stake: 1157130000000000000000000, // 1.15713 NEAR,
            public_key,
        },
    ))];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    // FIX: add actual obtained signature from speculos test somewhere in https://github.com/LedgerHQ/app-near/tree/develop/tests
    // on a per-actual-need basis
    let result_signature_from_speculos_test = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
