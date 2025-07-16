use near_ledger::NEARLedgerError;

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
            deploy_mode: near_primitives::action::GlobalContractDeployMode::CodeHash,
        },
    );
    tx.actions = vec![action];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    let result_signature_from_speculos_test = hex::decode("bf0108037f14e7f9284f566408f4ca345ed9adc297ed603a0297db9008dc0549a40d2448f1e2ce164c94fcc8a2db974174672bbe3b4fecd1e396665c97f25d03").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
