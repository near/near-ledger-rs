use near_ledger::NEARLedgerError;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);
    let public_key = common::get_static_secp256k1_public_key();
    let method_names = vec![
        "first_method",
        "saturating_add_signed",
        "iterator_chain_to_do_multiple_instances_of_an_operation_that_can_fail",
        "from_residual",
        "from_output",
        "unwrap_err_unchecked",
        "try_reserve_exact",
        "first_method",
        "saturating_add_signed",
        "iterator_chain_to_do_multiple_instances_of_an_operation_that_can_fail",
    ]
    .into_iter()
    .map(Into::into)
    .collect::<Vec<_>>();

    let permission = near_primitives_core::account::FunctionCallPermission {
        allowance: Some(150000000000000000000),
        receiver_id:
        "dc7e34eecec3096a4a661e10932834f801149c49dba9b93322f6d9de18047f9c1b11b3b31673033936ad07bddc01f9da27d974811e480fb197c799e23480a489".into(),
        method_names,
    };

    tx.actions = vec![near_primitives::transaction::Action::AddKey(Box::new(
        near_primitives::transaction::AddKeyAction {
            public_key,
            access_key: near_primitives_core::account::AccessKey {
                nonce: 127127127127,
                permission: near_primitives_core::account::AccessKeyPermission::FunctionCall(
                    permission,
                ),
            },
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
