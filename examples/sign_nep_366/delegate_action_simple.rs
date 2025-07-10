use std::{convert::TryInto, str::FromStr};

use clap::Parser;
use common::{static_speculos_public_key, ExampleArgs, StaticTestCase};
use near_account_id::AccountId;
use near_crypto::Signature;
use near_ledger::NEARLedgerError;
use near_primitives::action::delegate::{DelegateAction, SignedDelegateAction};
use slipped10::BIP32Path;

use crate::common::display_pub_key;

#[path = "../common/lib.rs"]
mod common;

// TODO #A0: extract common code for delegate action flow into ../common/lib.rs, make it an actions vec param, and expected speculos signature param
// TODO #A1: run, check that signature isn't changed

fn main() -> Result<(), NEARLedgerError> {
    env_logger::builder().init();
    let args = ExampleArgs::parse();

    // signature taken from https://github.com/LedgerHQ/app-near/blob/fc6c7e2cd0349cbfde938d9de2a92cfeb0d98a7d/tests/test_sign_nep366_delegate_action/test_nep366_delegate_action.py#L49
    let result_signature_from_speculos_test = hex::decode("c6645407278a472641350472fc83eb8002ef961ecf67102df5976adb5a071208db7309975dc0a56f7c5b604ea45ccfdf3d0a78be221c4afcee6aae03d394690c").unwrap();

    let maybe_static_test_case = if args.speculos_test_generate {
        Some(StaticTestCase {
            public_key: static_speculos_public_key(),
            expected_signature_bytes: result_signature_from_speculos_test,
        })
    } else {
        None
    };

    let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
    let ledger_pub_key = if let Some(ref static_test_case) = maybe_static_test_case {
        static_test_case.public_key.clone()
    } else {
        near_ledger::get_public_key_with_display_flag(hd_path.clone(), false)?
    };
    display_pub_key(ledger_pub_key);

    let sender_id = AccountId::from_str("bob.near").unwrap();

    let actions = vec![near_primitives::transaction::Action::Transfer(
        near_primitives::transaction::TransferAction {
            deposit: 150000000000000000000000, // 0.15 NEAR
        },
    )]
    .into_iter()
    .map(|action| action.try_into().unwrap())
    .collect::<Vec<_>>();

    let ledger_pub_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
        ledger_pub_key.to_bytes(),
    ));

    let delegate_action = DelegateAction {
        sender_id,
        receiver_id: AccountId::from_str("alice.near").unwrap(),
        actions,
        nonce: 127127122121,
        max_block_height: 100500,
        public_key: ledger_pub_key,
    };

    let bytes = borsh::to_vec(&delegate_action)
        .expect("Delegate action is not expected to fail on serialization");

    let (signature, signature_bytes) = if let Some(ref static_test_case) = maybe_static_test_case {
        near_ledger::print_apdus::nep366_delegate_action(&bytes, hd_path);
        let signature = Signature::from_parts(
            near_crypto::KeyType::ED25519,
            &static_test_case.expected_signature_bytes,
        )
        .unwrap();
        (signature, static_test_case.expected_signature_bytes.clone())
    } else {
        let signature_bytes = near_ledger::sign_message_nep366_delegate_action(&bytes, hd_path)?;

        let signature =
            Signature::from_parts(near_crypto::KeyType::ED25519, &signature_bytes).unwrap();
        (signature, signature_bytes)
    };

    let signed_delegate = SignedDelegateAction {
        delegate_action,
        signature: signature.clone(),
    };
    log::info!("{:#?}", signed_delegate);
    assert!(signed_delegate.verify());

    common::display_signature(signature_bytes);
    Ok(())
}
