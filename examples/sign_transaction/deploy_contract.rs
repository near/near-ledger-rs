use near_ledger::NEARLedgerError;
use near_primitives::transaction::DeployContractAction;
use near_primitives_core::hash::CryptoHash;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);

    let code = std::iter::repeat_n(42u8, 2000).collect::<Vec<_>>();

    let code_hash = CryptoHash::hash_bytes(&code);
    log::info!("Contract code hash: {code_hash:?}");
    let action =
        near_primitives::transaction::Action::DeployContract(DeployContractAction { code });
    log::warn!(
        "action bytes: {:x?}",
        borsh::to_vec(&action).expect("no ser err")
    );
    tx.actions = vec![action];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    let result_signature_from_speculos_test = hex::decode("d48d750cfc84fff62801dbd1e4899df3471b379dbba41decf38854c2c99971bba2256d77d6318f704a4c3351f692f85f78214f5e871500523de8698a3a7d9806").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
