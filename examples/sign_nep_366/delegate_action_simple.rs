use std::convert::TryInto;

use near_ledger::NEARLedgerError;

use crate::common::get_key_sign_nep_366_and_verify_flow_with_cli_parse;

#[path = "../common/lib.rs"]
mod common;

fn main() -> Result<(), NEARLedgerError> {
    // signature taken from https://github.com/LedgerHQ/app-near/blob/fc6c7e2cd0349cbfde938d9de2a92cfeb0d98a7d/tests/test_sign_nep366_delegate_action/test_nep366_delegate_action.py#L49
    let result_signature_from_speculos_test = hex::decode("c6645407278a472641350472fc83eb8002ef961ecf67102df5976adb5a071208db7309975dc0a56f7c5b604ea45ccfdf3d0a78be221c4afcee6aae03d394690c").unwrap();
    let actions = vec![near_primitives::transaction::Action::Transfer(
        near_primitives::transaction::TransferAction {
            deposit: 150000000000000000000000, // 0.15 NEAR
        },
    )]
    .into_iter()
    .map(|action| action.try_into().unwrap())
    .collect::<Vec<_>>();

    get_key_sign_nep_366_and_verify_flow_with_cli_parse(
        actions,
        result_signature_from_speculos_test,
    )
}
