#![allow(unused)]
use std::str::FromStr;

use ed25519_dalek::Signature;
use ed25519_dalek::Verifier;
use near_ledger::NEARLedgerError;
use near_primitives_core::{borsh, borsh::BorshSerialize, hash::CryptoHash, types::AccountId};
use slipped10::BIP32Path;

use clap::Parser;
use near_crypto::SecretKey;
use near_primitives::transaction::{DeployContractAction, FunctionCallAction};
pub fn display_pub_key(public_key: ed25519_dalek::VerifyingKey) {
    log::info!("---");
    log::info!("Public key:");
    log::info!("{:?}", public_key);
    log::info!("{:<10} : {}", "hex", hex::encode(public_key));
    log::info!(
        "{:<10} : {}",
        "base58",
        near_crypto::PublicKey::ED25519(
            near_crypto::ED25519PublicKey::from(public_key.to_bytes(),)
        )
    );
    log::info!("---");
}

pub fn tx_template(
    ledger_pub_key: ed25519_dalek::VerifyingKey,
) -> near_primitives::transaction::TransactionV0 {
    let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
        ledger_pub_key.to_bytes(),
    ));
    let block_hash = "Cb3vKNiF3MUuVoqfjuEFCgSNPT79pbuVfXXd2RxDXc5E"
        .parse::<CryptoHash>()
        .unwrap();

    let signer_account_str = hex::encode(ledger_pub_key.to_bytes());
    let receiver_account_str = "dc7e34eecec3096a4a661e10932834f801149c49dba9b93322f6d9de18047f9c";

    near_primitives::transaction::TransactionV0 {
        public_key,
        block_hash,
        nonce: 103595482000005,
        signer_id: AccountId::from_str(&signer_account_str).unwrap(),
        receiver_id: AccountId::from_str(receiver_account_str).unwrap(),
        actions: vec![],
    }
}

pub fn get_static_secp256k1_public_key() -> near_crypto::PublicKey {
    near_crypto::PublicKey::from_str("secp256k1:2xV3hzGShUE3X5jE9jmAyFC67GfgwAUo5FoBJ79Zh84Z5Ubdxy94Ka73EWwrFg5FbVYAvtdqJK77P6CAdyMkEnca").expect("no parse err")
}

pub fn get_static_ed25519_key() -> near_crypto::PublicKey {
    near_crypto::PublicKey::from_str("ed25519:AVHQN9NRrHGeagz7RiVEUyhB9aiSGZCJbXKbJbW8z63E")
        .expect("no parse err")
}

#[allow(deprecated)]
pub fn batch_of_all_types_of_actions_v1(
    ledger_pub_key: ed25519_dalek::VerifyingKey,
) -> Vec<near_primitives::transaction::Action> {
    let create_account = near_primitives::transaction::Action::CreateAccount(
        near_primitives::transaction::CreateAccountAction {},
    );

    let delete_account = near_primitives::transaction::Action::DeleteAccount(
        near_primitives::transaction::DeleteAccountAction {
            beneficiary_id: AccountId::new_unvalidated(
                "dc7e34eecec3096a4a661e10932834f801149c49dba9b93322f6d9de18047f9c1b11b3b31673033936ad07bddc01f9da27d974811e480fb197c799e23480a489".to_string()),
        },
    );

    let delete_key_ed25519 = {
        let public_key_ed = get_static_ed25519_key();
        near_primitives::transaction::Action::DeleteKey(Box::new(
            near_primitives::transaction::DeleteKeyAction {
                public_key: public_key_ed,
            },
        ))
    };

    let delete_key_secp256k1 = near_primitives::transaction::Action::DeleteKey(Box::new(
        near_primitives::transaction::DeleteKeyAction {
            public_key: get_static_secp256k1_public_key(),
        },
    ));

    let stake = near_primitives::transaction::Action::Stake(Box::new(
        near_primitives::transaction::StakeAction {
            stake: 1157130000000000000000000, // 1.15713 NEAR,
            public_key: get_static_secp256k1_public_key(),
        },
    ));

    let add_key_fullaccess = near_primitives::transaction::Action::AddKey(Box::new(
        near_primitives::transaction::AddKeyAction {
            public_key: get_static_secp256k1_public_key(),
            access_key: near_primitives_core::account::AccessKey {
                nonce: 127127127127,
                permission: near_primitives_core::account::AccessKeyPermission::FullAccess,
            },
        },
    ));

    let add_key_function_call = {
        let permission = {
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
            near_primitives_core::account::FunctionCallPermission {
                    allowance: Some(150000000000000000000),
                    receiver_id:
                    "dc7e34eecec3096a4a661e10932834f801149c49dba9b93322f6d9de18047f9c1b11b3b31673033936ad07bddc01f9da27d974811e480fb197c799e23480a489".into(),
                    method_names,
                }
        };
        near_primitives::transaction::Action::AddKey(Box::new(
            near_primitives::transaction::AddKeyAction {
                public_key: get_static_secp256k1_public_key(),
                access_key: near_primitives_core::account::AccessKey {
                    nonce: 127127127127,
                    permission: near_primitives_core::account::AccessKeyPermission::FunctionCall(
                        permission,
                    ),
                },
            },
        ))
    };

    let transfer = near_primitives::transaction::Action::Transfer(
        near_primitives::transaction::TransferAction {
            deposit: 150000000000000000000000, // 0.15 NEAR
        },
    );

    let deploy_contract = {
        let code = core::iter::repeat(42u8).take(30).collect::<Vec<_>>();

        let code_hash = CryptoHash::hash_bytes(&code);
        log::info!("Contract code hash: {:?}", code_hash);
        near_primitives::transaction::Action::DeployContract(DeployContractAction { code })
    };

    let function_call_str_args = {
        let args_str = r#"{"previous_vesting_schedule_with_salt":{"vesting_schedule":{"start_timestamp":"1577919600000000000","cliff_timestamp":"1609455600000000000","end_timestamp":"1704150000000000000"},"salt":"7bc709c22801118b743fae3866edb4dea1630a97ab9cd67e993428b94a0f397a"}, "vesting_schedule_with_salt":{"vesting_schedule":{"start_timestamp":"1577919600000000000","cliff_timestamp":"1609455600000000000","end_timestamp":"1704150000000000000"},"salt":"7bc709c22801118b743fae3866edb4dea1630a97ab9cd67e993428b94a0f397aababab"}}"#;

        let f_call = FunctionCallAction {
            method_name: "saturating_add_signed".to_string(),
            args: args_str.as_bytes().to_vec(),
            gas: 127127122121,
            deposit: 150000000000000000000000, // 0.15 NEAR,
        };
        near_primitives::transaction::Action::FunctionCall(Box::new(f_call))
    };

    let function_call_binary_args = {
        let args_binary = hex::decode("204f6e206f6c646572207465726d696e616c732c2074686520756e64657273636f726520636f646520697320646973706c617965642061732061206c6566740a202020202020206172726f772c2063616c6c6564206261636b6172726f772c2074686520636172657420697320646973706c6179656420617320616e2075702d6172726f770a20202020202020616e642074686520766572746963616c2062617220686173206120686f6c6520696e20746865206d6964646c652e0a0a2020202020202055707065726361736520616e64206c6f77657263617365206368617261637465727320646966666572206279206a757374206f6e652062697420616e64207468650a20202020202020415343494920636861726163746572203220646966666572732066726f6d2074686520646f75626c652071756f7465206279206a757374206f6e65206269742c0a20202020202020746f6f2e202054686174206d616465206974206d7563682065617369657220746f20656e636f64652063686172616374657273206d656368616e6963616c6c790a202020202020206f7220776974682061206e6f6e2d6d6963726f636f6e74726f6c6c65722d626173656420656c656374726f6e6963206b6579626f61726420616e6420746861740a2020202020202070616972696e672077617320666f756e64206f6e206f6c642074656c6574797065732e0a").unwrap();

        let f_call = FunctionCallAction {
            method_name: "saturating_add_signed".to_string(),
            args: args_binary,
            gas: 127127122121,
            deposit: 150000000000000000000000, // 0.15 NEAR,
        };
        near_primitives::transaction::Action::FunctionCall(Box::new(f_call))
    };

    let function_call_binary_args_after_parse_error = {
        let mut bytes = vec![];
        bytes.push(123u8);

        bytes.extend((0..255).collect::<Vec<_>>());

        let f_call = FunctionCallAction {
            method_name: "saturating_add_signed".to_string(),
            args: bytes,
            gas: 127127122121,
            deposit: 150000000000000000000000, // 0.15 NEAR,
        };

        near_primitives::transaction::Action::FunctionCall(Box::new(f_call))
    };

    vec![
        create_account,
        delete_account,
        delete_key_ed25519,
        delete_key_secp256k1,
        stake,
        add_key_fullaccess,
        add_key_function_call,
        transfer,
        deploy_contract,
        function_call_str_args,
        function_call_binary_args,
        function_call_binary_args_after_parse_error,
    ]
}

pub fn batch_of_all_types_of_actions_v2(
    ledger_pub_key: ed25519_dalek::VerifyingKey,
) -> Vec<near_primitives::transaction::Action> {
    let mut v1_vector = batch_of_all_types_of_actions_v1(ledger_pub_key.clone());
    let deploy_global_as_hash = {
        let code = core::iter::repeat(42u8).take(3000).collect::<Vec<_>>();
        near_primitives::transaction::Action::DeployGlobalContract(
            near_primitives::action::DeployGlobalContractAction {
                code: std::sync::Arc::from(code),
                deploy_mode: near_primitives::action::GlobalContractDeployMode::CodeHash,
            },
        )
    };
    let deploy_global_as_acc = {
        let code = core::iter::repeat(42u8).take(3000).collect::<Vec<_>>();
        near_primitives::transaction::Action::DeployGlobalContract(
            near_primitives::action::DeployGlobalContractAction {
                code: std::sync::Arc::from(code),
                deploy_mode: near_primitives::action::GlobalContractDeployMode::AccountId,
            },
        )
    };
    let use_global_by_hash = {
        let referenced_contract_hash = "5KaX9FM9NtjpfahksL8TMWQk3LF7k8Sv88Qem4tGrVDW"
            .parse::<CryptoHash>()
            .unwrap();
        near_primitives::transaction::Action::UseGlobalContract(Box::new(
            near_primitives::action::UseGlobalContractAction {
                contract_identifier: near_primitives::action::GlobalContractIdentifier::CodeHash(
                    referenced_contract_hash,
                ),
            },
        ))
    };
    let use_global_by_acc = {
        let referenced_account_id =
            near_primitives::types::AccountId::from_str("simple-package-verify-rs-ci.testnet")
                .unwrap();
        near_primitives::transaction::Action::UseGlobalContract(Box::new(
            near_primitives::action::UseGlobalContractAction {
                contract_identifier: near_primitives::action::GlobalContractIdentifier::AccountId(
                    referenced_account_id,
                ),
            },
        ))
    };
    v1_vector.extend([
        deploy_global_as_hash,
        deploy_global_as_acc,
        use_global_by_hash,
        use_global_by_acc,
    ]);
    v1_vector
}

pub fn serialize_and_display_tx(transaction: near_primitives::transaction::Transaction) -> Vec<u8> {
    log::info!("---");
    log::info!("Transaction:");
    log::info!("{:#?}", transaction);
    let bytes =
        borsh::to_vec(&transaction).expect("Transaction is not expected to fail on serialization");
    log::info!("transaction byte array length: {}", bytes.len());
    log::info!("---");
    bytes
}
pub fn display_signature(signature_bytes: Vec<u8>) -> ed25519_dalek::Signature {
    log::info!("---");
    log::info!("Signature:");
    let signature = Signature::from_slice(&signature_bytes).unwrap();

    let signature_near =
        near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature_bytes)
            .expect("Signature is not expected to fail on deserialization");
    log::info!("{:<20} : {}", "signature (hex)", signature);
    log::info!("{:<20} : {}", "signature (base58)", signature_near);
    signature
}

pub fn display_and_verify_signature(
    msg: Vec<u8>,
    signature_bytes: Vec<u8>,
    public_key: ed25519_dalek::VerifyingKey,
) {
    let signature = display_signature(signature_bytes);
    assert!(public_key
        .verify(CryptoHash::hash_bytes(&msg).as_ref(), &signature)
        .is_ok());
    log::info!("---");
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ExampleArgs {
    #[clap(long, short, action)]
    pub speculos_test_generate: bool,
}

pub fn get_key_sign_and_verify_flow_with_cli_parse<F>(
    f_transaction: F,
    expected_signature_bytes: Vec<u8>,
) -> Result<(), NEARLedgerError>
where
    F: FnOnce(ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction,
{
    let args = ExampleArgs::parse();

    let maybe_static_test_case = if args.speculos_test_generate {
        Some(StaticTestCase {
            public_key: static_speculos_public_key(),
            expected_signature_bytes,
        })
    } else {
        None
    };
    get_key_sign_and_verify_flow(f_transaction, maybe_static_test_case)
}

pub fn get_key_sign_and_verify_flow<F>(
    f_transaction: F,
    maybe_static_test_case: Option<StaticTestCase>,
) -> Result<(), NEARLedgerError>
where
    F: FnOnce(ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction,
{
    env_logger::builder().init();
    let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();

    let ledger_pub_key = if let Some(ref static_test_case) = maybe_static_test_case {
        static_test_case.public_key.clone()
    } else {
        near_ledger::get_public_key_with_display_flag(hd_path.clone(), false)?
    };

    display_pub_key(ledger_pub_key);

    let unsigned_transaction = f_transaction(ledger_pub_key);

    let bytes = serialize_and_display_tx(unsigned_transaction);

    if let Some(ref static_test_case) = maybe_static_test_case {
        near_ledger::print_apdus::transaction(&bytes, hd_path);
        display_and_verify_signature(
            bytes,
            static_test_case.expected_signature_bytes.clone(),
            ledger_pub_key,
        );
    } else {
        let signature_bytes = near_ledger::sign_transaction(&bytes, hd_path)?;
        display_and_verify_signature(bytes, signature_bytes, ledger_pub_key);
    }

    Ok(())
}

pub struct StaticTestCase {
    pub public_key: ed25519_dalek::VerifyingKey,
    pub expected_signature_bytes: Vec<u8>,
}

pub fn static_speculos_public_key() -> ed25519_dalek::VerifyingKey {
    let bytes =
        hex::decode("c4f5941e81e071c2fd1dae2e71fd3d859d462484391d9a90bf219211dcbb320f").unwrap();
    let buffer = {
        let mut buffer = [0u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
        buffer.copy_from_slice(&bytes);
        buffer
    };

    ed25519_dalek::VerifyingKey::from_bytes(&buffer).unwrap()
}
