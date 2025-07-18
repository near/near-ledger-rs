use near_ledger::NEARLedgerError;
use near_primitives_core::hash::CryptoHash;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);

    let referenced_contract_hash = "5KaX9FM9NtjpfahksL8TMWQk3LF7k8Sv88Qem4tGrVDW"
        .parse::<CryptoHash>()
        .unwrap();

    log::warn!("referenced_contract_hash: {}", referenced_contract_hash);
    let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
        near_primitives::action::UseGlobalContractAction {
            contract_identifier: near_primitives::action::GlobalContractIdentifier::CodeHash(
                referenced_contract_hash,
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
    let result_signature_from_speculos_test = hex::decode("4bd819a0bfa5b49324bf2dae0dfeb8b135a7973682acd01a830e3a550a6ef6c1b51896f7c6101c5cc3efcd3832e4c994cc90377eee69862a6a786b081673da01").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)
}
