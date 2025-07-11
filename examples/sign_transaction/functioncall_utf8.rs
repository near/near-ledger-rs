use near_ledger::NEARLedgerError;
use near_primitives::transaction::FunctionCallAction;

#[path = "../common/lib.rs"]
mod common;

fn tx(ledger_pub_key: ed25519_dalek::VerifyingKey) -> near_primitives::transaction::Transaction {
    let mut tx = common::tx_template(ledger_pub_key);

    // 2, 3 and 4 bytes
    let utf_mutibyte_chars = [('©', 2), ('ଔ', 3), ('🝙', 4)];

    let mut str = String::new();

    for i in 0..100 {
        let (char, bytes) = utf_mutibyte_chars[i % 3];
        str.push_str(&format!("{bytes}"));
        str.push(char);
    }

    let args = format!("{{\"test_utf8_key\": \"{str}\"}}");

    println!("args: {args}");
    println!("args.len() {}", args.len());

    let f_call = FunctionCallAction {
        method_name: "test_payload_with_utf8_text".to_string(),
        args: args.as_bytes().to_vec(),
        gas: 127127122121,
        deposit: 150000000000000000000000, // 0.15 NEAR,
    };

    println!("{args:?}");

    tx.actions = vec![near_primitives::transaction::Action::FunctionCall(
        Box::new(f_call),
    )];
    near_primitives::transaction::Transaction::V0(tx)
}

fn main() -> Result<(), NEARLedgerError> {
    // FIX: add actual obtained signature from speculos test somewhere in https://github.com/LedgerHQ/app-near/tree/develop/tests
    // on a per-actual-need basis
    let result_signature_from_speculos_test = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

    common::get_key_sign_and_verify_flow_with_cli_parse(tx, result_signature_from_speculos_test)?;
    Ok(())
}
