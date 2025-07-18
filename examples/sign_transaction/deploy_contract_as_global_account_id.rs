use near_ledger::NEARLedgerError;
use near_primitives_core::hash::CryptoHash;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);

    let code = std::iter::repeat_n(42u8, 3000).collect::<Vec<_>>();

    let code_hash = CryptoHash::hash_bytes(&code);
    log::warn!("Contract code hash: {code_hash:?}");
    let action = near_primitives::transaction::Action::DeployGlobalContract(
        near_primitives::action::DeployGlobalContractAction {
            code: std::sync::Arc::from(code),
            deploy_mode: near_primitives::action::GlobalContractDeployMode::AccountId,
        },
    );
    log::warn!(
        "action bytes: {:x?}",
        borsh::to_vec(&action).expect("no ser err")
    );
    tx.actions = vec![action];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    let result_signature_from_speculos_test = hex::decode("799cd18788012b5209692f4aaaeae951393f80011abf564457a5d5a6fe7456870e3df05591ef592f03a563428cf3feb47170152e3acc1ac305f49a74359da90a").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
