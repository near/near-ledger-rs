# near-ledger-rs

[![Rust](https://github.com/khorolets/near-ledger-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/khorolets/near-ledger-rs/actions/workflows/rust.yml)
[![](http://meritbadge.herokuapp.com/near-ledger)](https://crates.io/crates/near-ledger)
[![]( https://docs.rs/near-ledger/badge.svg)]( https://docs.rs/near-ledger/)

It is NEAR <-> Ledger transport


Provides a set of commands that can be executed to communicate with NEAR App installed on Ledger device:

* Read PublicKey from Ledger device by HD Path
* Sign a Transaction


## Examples


### Get PublicKey from Ledger


```rust
use near_ledger::get_public_key;
use slipped10::BIP32Path;
use std::str::FromStr;

let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
let public_key = get_public_key(hd_path).unwrap();
println!("{:#?}", public_key);
```


#### Trick


To convert the answer into `near_crypto::PublicKey` do:

```rust
let public_key = near_crypto::PublicKey::ED25519(
    near_crypto::ED25519PublicKey::from(
        public_key.to_bytes(),
    )
);
```


### How to sign a transaction


```rust
use near_ledger::{sign_transaction, SignTarget};
use near_primitives::borsh::BorshSerialize;
use slipped10::BIP32Path;
use std::str::FromStr;

let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
let borsh_transaction = near_unsigned_transaction.try_to_vec().unwrap();
let signature = sign_transaction(SignTarget::BorshUnsignedTx(borsh_transaction), hd_path).unwrap();
println!("{:#?}", signature);
```


#### Trick

To convert the answer into `near_crypto::Signature` do:


```rust
let signature = near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
    .expect("Signature is not expected to fail on deserialization");
```

## Executable examples

### Get version

```bash
RUST_LOG=near_ledger=info cargo run --example get_version
```

### Get PublicKey from Ledger

#### Display

```bash
RUST_LOG=near_ledger=info cargo run --example get_public_key_display
```
#### Silent

```bash
RUST_LOG=near_ledger=info cargo run --example get_public_key_silent
```

### Get WalletID from Ledger

```bash
RUST_LOG=near_ledger=info cargo run --example get_wallet_id
```
### Sign a transaction

All all following examples have `--speculos-test-generate` cli flag which just generates APDUs
to be sent without interacting with an actual device.

#### Transfer

```bash
RUST_LOG=near_ledger=info cargo run --example sign_transfer
RUST_LOG=near_ledger=info cargo run --example sign_transfer  -- --speculos-test-generate
```

#### Other

```bash
export RUST_LOG=near_ledger=info
# doing 
# export RUST_LOG=info 
# or 
# export RUST_LOG=near_ledger=info,sign_create_account=info
# will print additional info about public key/transaction 
cargo run --example sign_create_account
cargo run --example sign_create_account  -- --speculos-test-generate
cargo run --example sign_delete_account_short
cargo run --example sign_delete_account_short  -- --speculos-test-generate
cargo run --example sign_delete_account_long
cargo run --example sign_delete_account_long  -- --speculos-test-generate
cargo run --example sign_delete_key_ed25519
cargo run --example sign_delete_key_ed25519  -- --speculos-test-generate
cargo run --example sign_delete_key_secp256k1
cargo run --example sign_delete_key_secp256k1  -- --speculos-test-generate
cargo run --example sign_stake
cargo run --example sign_stake  -- --speculos-test-generate
cargo run --example sign_add_key_fullaccess
cargo run --example sign_add_key_fullaccess  -- --speculos-test-generate
cargo run --example sign_add_key_functioncall
cargo run --example sign_add_key_functioncall  -- --speculos-test-generate
cargo run --example sign_deploy_contract
cargo run --example sign_deploy_contract  -- --speculos-test-generate
cargo run --example sign_functioncall_str
cargo run --example sign_functioncall_str  -- --speculos-test-generate
cargo run --example functioncall_str_newline
cargo run --example functioncall_str_newline  -- --speculos-test-generate
cargo run --example functioncall_str_ascii_subrange
cargo run --example functioncall_str_ascii_subrange  -- --speculos-test-generate
cargo run --example functioncall_utf8
cargo run --example functioncall_utf8  -- --speculos-test-generate
cargo run --example sign_functioncall_bin
cargo run --example sign_functioncall_bin  -- --speculos-test-generate
cargo run --example sign_functioncall_str_parse_err
cargo run --example sign_functioncall_str_parse_err -- --speculos-test-generate
cargo run --example sign_batch_all_actions_v1
cargo run --example sign_batch_all_actions_v1  -- --speculos-test-generate
cargo run --example sign_deploy_contract_as_global_hash
cargo run --example sign_deploy_contract_as_global_hash  -- --speculos-test-generate
cargo run --example sign_deploy_contract_as_global_account_id
cargo run --example sign_deploy_contract_as_global_account_id  -- --speculos-test-generate
```


<!-- TODO #A: add deploy_contract_use_global_hash -->

<!-- TODO #A: add deploy_contract_use_global_account_id -->

### Sign a NEP-413 message

```bash
RUST_LOG=near_ledger=info cargo run --example sign_nep_413_message
RUST_LOG=near_ledger=info cargo run --example sign_nep_413_message  -- --speculos-test-generate
```

### Sign a NEP-366 delegate action

```bash
RUST_LOG=near_ledger=info cargo run --example sign_nep_366_delegate_action_batch_all_sub_actions_v1
RUST_LOG=near_ledger=info cargo run --example sign_nep_366_delegate_action_batch_all_sub_actions_v1  -- --speculos-test-generate
RUST_LOG=near_ledger=info cargo run --example sign_nep_366_delegate_action_simple
RUST_LOG=near_ledger=info cargo run --example sign_nep_366_delegate_action_simple  -- --speculos-test-generate
```

### Open near app

```bash
RUST_LOG=near_ledger=info cargo run --package near-ledger --example open_application 
```
