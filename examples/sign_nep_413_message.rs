use std::str::FromStr;

use clap::Parser;
use common::static_speculos_public_key;
use common::ExampleArgs;
use common::StaticTestCase;
use ed25519_dalek::Signature;
use ed25519_dalek::Verifier;
use near_ledger::{NEARLedgerError, NEP413Payload};
use near_primitives::signable_message::{MessageDiscriminant, SignableMessage};
use near_primitives_core::{borsh, hash::CryptoHash};
use near_slip10::BIP32Path;

use crate::common::display_pub_key;

#[path = "./common/lib.rs"]
mod common;

pub fn display_and_verify_signature(
    msg: &NEP413Payload,
    signature_bytes: Vec<u8>,
    public_key: ed25519_dalek::VerifyingKey,
) {
    log::info!("---");
    log::info!("Signature:");
    let signature = Signature::from_slice(&signature_bytes).unwrap();

    let msg_discriminant = MessageDiscriminant::new_off_chain(413).unwrap();
    let signable_message = SignableMessage {
        discriminant: msg_discriminant,
        msg,
    };

    let hash = CryptoHash::hash_bytes(&borsh::to_vec(&signable_message).unwrap());

    let signature_near =
        near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature_bytes)
            .expect("Signature is not expected to fail on deserialization");
    log::info!("{:<20} : {}", "signature (hex)", signature);
    log::info!("{:<20} : {}", "signature (base58)", signature_near);

    assert!(public_key.verify(hash.as_ref(), &signature).is_ok());
    log::info!("---");
}

fn main() -> Result<(), NEARLedgerError> {
    env_logger::builder().init();
    let args = ExampleArgs::parse();

    // signature taken from https://github.com/LedgerHQ/app-near/blob/fc6c7e2cd0349cbfde938d9de2a92cfeb0d98a7d/tests/test_sign_nep413_msg/test_nep413_msg.py#L82
    let result_signature_from_speculos_test = hex::decode("eb1200a990ba295ebd3b5a49729a30734179d2414cb43bd8af39b7103ac4dcdfd3174409a434a1f6a48d267e4f46492886129343076f8315afaf9e761183490e").unwrap();

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

    let msg = NEP413Payload {
        message: "Makes it possible to authenticate users without having to add new access keys. This will improve UX, save money and will not increase the on-chain storage of the users' accounts./Makes it possible to authenticate users without having to add new access keys. This will improve UX, save money and will not increase the on-chain storage of the users' accounts./Makes it possible to authenticate users without having to add new access keys. This will improve UX, save money and will not increase the on-chain storage of the users' accounts.".to_string(),
        nonce: [42; 32],
        recipient: "alice.near".to_string(),
        callback_url: Some("myapp.com/callback".to_string()) 
    };
    if let Some(ref static_test_case) = maybe_static_test_case {
        near_ledger::print_apdus::message_nep413(&msg, hd_path);
        display_and_verify_signature(
            &msg,
            static_test_case.expected_signature_bytes.clone(),
            ledger_pub_key,
        );
    } else {
        let signature_bytes = near_ledger::sign_message_nep413(&msg, hd_path)?;

        display_and_verify_signature(&msg, signature_bytes, ledger_pub_key);
    }

    Ok(())
}
