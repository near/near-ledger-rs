//! NEAR <-> Ledger transport
//!
//! Provides a set of commands that can be executed to communicate with NEAR App installed on Ledger device:
//! - Read PublicKey from Ledger device by HD Path
//! - Sign a Transaction
use std::{thread::sleep, time::Duration};

use borsh::BorshSerialize;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use ledger_apdu::APDUAnswer;
use ledger_transport::APDUCommand;
use ledger_transport_hid::{
    hidapi::{HidApi, HidError},
    LedgerHIDError, TransportNativeHID,
};

pub mod print_apdus;

#[cfg(feature = "ble")]
mod ble;
#[cfg(feature = "ble")]
mod ble_api;

#[cfg(feature = "ble")]
pub use ble::{BleError, TransportBle};

#[cfg(feature = "ble")]
pub use ble_api::{
    get_public_key_ble, get_public_key_with_display_flag_ble, get_version_ble, get_wallet_id_ble,
    open_near_application_ble, sign_message_nep366_delegate_action_ble, sign_message_nep413_ble,
    sign_transaction_ble,
};

pub(crate) const CLA: u8 = 0x80; // Instruction class
pub(crate) const INS_GET_PUBLIC_KEY: u8 = 4; // Instruction code to get public key
pub(crate) const INS_GET_WALLET_ID: u8 = 0x05; // Get Wallet ID
pub(crate) const INS_GET_VERSION: u8 = 6; // Instruction code to get app version from the Ledger
pub(crate) const INS_SIGN_TRANSACTION: u8 = 2; // Instruction code to sign a transaction on the Ledger
pub(crate) const INS_SIGN_NEP413_MESSAGE: u8 = 7; // Instruction code to sign a nep-413 message with Ledger
pub(crate) const INS_SIGN_NEP366_DELEGATE_ACTION: u8 = 8; // Instruction code to sign a nep-413 message with Ledger
pub(crate) const NETWORK_ID: u8 = b'W'; // Instruction parameter 2
pub(crate) const RETURN_CODE_OK: u16 = 0x9000; // APDUAnswer.retcode which means success from Ledger
pub(crate) const CHUNK_SIZE: usize = 250; // Chunk size to be sent to Ledger

// Dashboard APDU constants (BOLOS / device-level commands)
pub(crate) const CLA_DASHBOARD: u8 = 0xB0;
pub(crate) const INS_GET_APP_NAME: u8 = 0x01;
pub(crate) const INS_QUIT_APP: u8 = 0xA7;
pub(crate) const CLA_OPEN_APP: u8 = 0xE0;
pub(crate) const INS_OPEN_APP: u8 = 0xD8;

pub(crate) const RETURN_CODE_APP_MISSING: u16 = 0x6807;
pub(crate) const RETURN_CODE_ERROR_INPUT: u16 = 0x670A;
pub(crate) const RETURN_CODE_UNKNOWN_ERROR: u16 = 0x5515;

/// This error code is returned when the user declines to open the app.
/// But I couldn't find it in the any of the ledger documentation...
pub(crate) const RETURN_CODE_DECLINE: u16 = 0x5501;

/// ISO 7816 "CLA not supported" — typically means the NEAR app is not open
pub(crate) const RETURN_CODE_CLA_NOT_SUPPORTED: u16 = 0x6E01;
/// ISO 7816 "INS not supported"
pub(crate) const RETURN_CODE_INS_NOT_SUPPORTED: u16 = 0x6D00;

/// Map a Ledger APDU return code to a human-readable error string.
pub(crate) fn retcode_to_error_string(retcode: u16) -> String {
    match retcode {
        RETURN_CODE_OK => "Success".to_string(),
        RETURN_CODE_CLA_NOT_SUPPORTED => {
            "NEAR app is not open on the Ledger device (CLA not supported, 0x6E01)".to_string()
        }
        RETURN_CODE_INS_NOT_SUPPORTED => {
            "Instruction not supported by the Ledger app (INS not supported, 0x6D00)".to_string()
        }
        RETURN_CODE_APP_MISSING => {
            "NEAR application is missing on the Ledger device (0x6807)".to_string()
        }
        RETURN_CODE_ERROR_INPUT => {
            "Internal error: the input length of bytes is not correct (0x670A)".to_string()
        }
        RETURN_CODE_DECLINE => {
            "User declined the request on the Ledger device (0x5501)".to_string()
        }
        RETURN_CODE_UNKNOWN_ERROR => "Unknown error from the Ledger device (0x5515)".to_string(),
        _ => format!("Ledger APDU retcode: 0x{:X}", retcode),
    }
}

/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type BorshSerializedUnsignedTransaction<'a> = &'a [u8];
/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type BorshSerializedDelegateAction<'a> = &'a [u8];

pub(crate) const P1_GET_PUB_DISPLAY: u8 = 0;
pub(crate) const P1_GET_PUB_SILENT: u8 = 1;

pub(crate) const P1_SIGN_NORMAL: u8 = 0;
pub(crate) const P1_SIGN_NORMAL_LAST_CHUNK: u8 = 0x80;

/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type NEARLedgerAppVersion = Vec<u8>;
/// Alias of `Vec<u8>`. The goal is naming to help understand what the bytes to deal with
pub type SignatureBytes = Vec<u8>;

#[derive(Debug)]
pub enum NEARLedgerError {
    /// Error occuring on init of hidapid and getting current devices list
    HidApiError(HidError),
    /// Error occuring on creating a new hid transport, connecting to first ledger device found  
    LedgerHidError(LedgerHIDError),
    /// Error occurred while exchanging with Ledger device
    APDUExchangeError(String),
    /// Error with transport
    LedgerHIDError(LedgerHIDError),
    /// Error with BLE transport (only available with `ble` feature)
    #[cfg(feature = "ble")]
    BleError(String),
}

/// Converts BIP32Path into bytes (`Vec<u8>`)
pub(crate) fn hd_path_to_bytes(hd_path: &near_slip10::BIP32Path) -> Vec<u8> {
    (0..hd_path.depth())
        .flat_map(|index| {
            let value = *hd_path.index(index).unwrap();
            value.to_be_bytes()
        })
        .collect::<Vec<u8>>()
}

#[inline(always)]
pub(crate) fn log_command(index: usize, is_last_chunk: bool, command: &APDUCommand<Vec<u8>>) {
    log::info!(
        "APDU  in{}: {}",
        if is_last_chunk {
            " (last)".to_string()
        } else {
            format!(" ({})", index)
        },
        hex::encode(command.serialize())
    );
}

/// Get the version of NEAR App installed on Ledger
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `NEARLedgerAppVersion` (just a `Vec<u8>` for now, where first value is a major version, second is a minor and the last is the path)
///   and whose `Err` value is a `NEARLedgerError` containing an error which occurred.
pub fn get_version() -> Result<NEARLedgerAppVersion, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = get_transport()?;
    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_VERSION,
        p1: 0, // Instruction parameter 1 (offset)
        p2: 0,
        data: vec![],
    };

    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => {
            log::info!(
                "APDU out: {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );
            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            if response.retcode() == RETURN_CODE_OK {
                Ok(response.data().to_vec())
            } else {
                Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    response.retcode(),
                )))
            }
        }
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

fn running_app_name() -> Result<String, NEARLedgerError> {
    let transport = get_transport()?;

    let command = APDUCommand {
        cla: CLA_DASHBOARD,
        ins: INS_GET_APP_NAME,
        p1: 0,
        p2: 0,
        data: vec![],
    };
    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => {
            log::info!(
                "APDU out: {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );

            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            match response.retcode() {
                RETURN_CODE_OK => {
                    // Output format:
                    // * format u8
                    // * ascii name length u8
                    // * name in ascii

                    let data = response.data();
                    let app_name_len = data[1] as usize;
                    let app_name = String::from_utf8_lossy(&data[2..2 + app_name_len]).to_string();

                    Ok(app_name)
                }
                RETURN_CODE_UNKNOWN_ERROR => Err(NEARLedgerError::APDUExchangeError(
                    "The ledger most likely is locked. Please unlock ledger or reconnect it"
                        .to_string(),
                )),
                retcode => Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    retcode,
                ))),
            }
        }
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

fn quit_open_application() -> Result<(), NEARLedgerError> {
    let transport = get_transport()?;

    let command = APDUCommand {
        cla: CLA_DASHBOARD,
        ins: INS_QUIT_APP,
        p1: 0,
        p2: 0,
        data: vec![],
    };

    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => {
            log::info!(
                "APDU out: {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );

            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            match response.retcode() {
                RETURN_CODE_OK => Ok(()),
                retcode => Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    retcode,
                ))),
            }
        }
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

/// Open the NEAR application on the Ledger device
///
/// This is needed to do before calling other NEAR application
/// related methods
pub fn open_near_application() -> Result<(), NEARLedgerError> {
    match running_app_name()?.as_str() {
        "NEAR" => return Ok(()),
        // BOLOS is a ledger dashboard
        "BOLOS" => {}
        _ => {
            quit_open_application()?;
            // It won't work if we don't wait for the Ledger to close the app
            sleep(Duration::from_secs(1));
        }
    }

    let transport = get_transport()?;
    let data = b"NEAR".to_vec();
    let command: APDUCommand<Vec<u8>> = APDUCommand {
        cla: CLA_OPEN_APP,
        ins: INS_OPEN_APP,
        p1: 0x00,
        p2: 0x00,
        data,
    };

    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => {
            log::info!("APDU ret code: {:x}", response.retcode(),);

            // Ok means we successfully exchanged with the Ledger
            // but doesn't mean our request succeeded
            // we need to check it based on `response.retcode`
            match response.retcode() {
                RETURN_CODE_OK => Ok(()),
                retcode => Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    retcode,
                ))),
            }
        }
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

/// Gets PublicKey from the Ledger on the given `hd_path`
///
/// # Inputs
/// * `hd_path` - seed phrase hd path `near_slip10::BIP32Path` for which PublicKey to look
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `ed25519_dalek::VerifyingKey` and whose `Err` value is a
///   `NEARLedgerError` containing an error which
///   occurred.
///
/// # Examples
///
/// ```no_run
/// use near_ledger::get_public_key;
/// use near_slip10::BIP32Path;
/// use std::str::FromStr;
///
/// # fn main() {
/// let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
/// let public_key = get_public_key(hd_path).unwrap();
/// println!("{:#?}", public_key);
/// # }
/// ```
///
/// # Trick
///
/// To convert the answer into `near_crypto::PublicKey` do:
///
/// ```
/// # let public_key_bytes = [10u8; 32];
/// # let public_key = ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes).unwrap();
/// let public_key = near_crypto::PublicKey::ED25519(
///     near_crypto::ED25519PublicKey::from(
///         public_key.to_bytes(),
///     )
/// );
/// ```
pub fn get_public_key(
    hd_path: near_slip10::BIP32Path,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    get_public_key_with_display_flag(hd_path, true)
}

pub fn get_public_key_with_display_flag(
    hd_path: near_slip10::BIP32Path,
    display_and_confirm: bool,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = get_transport()?;

    // hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&hd_path);

    let p1 = if display_and_confirm {
        P1_GET_PUB_DISPLAY
    } else {
        P1_GET_PUB_SILENT
    };

    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_PUBLIC_KEY,
        p1, // Instruction parameter 1 (offset)
        p2: NETWORK_ID,
        data: hd_path_bytes,
    };
    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => handle_public_key_response(response),
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

pub fn get_wallet_id(
    hd_path: near_slip10::BIP32Path,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let transport = get_transport()?;

    // hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&hd_path);

    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_WALLET_ID,
        p1: 0, // Instruction parameter 1 (offset)
        p2: NETWORK_ID,
        data: hd_path_bytes,
    };
    log::info!("APDU  in: {}", hex::encode(command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => handle_public_key_response(response),
        Err(err) => Err(NEARLedgerError::LedgerHIDError(err)),
    }
}

fn handle_public_key_response(
    response: APDUAnswer<Vec<u8>>,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    log::info!(
        "APDU out: {}\nAPDU ret code: {:x}",
        hex::encode(response.apdu_data()),
        response.retcode(),
    );
    // Ok means we successfully exchanged with the Ledger
    // but doesn't mean our request succeeded
    // we need to check it based on `response.retcode`
    if response.retcode() == RETURN_CODE_OK {
        let data = response.data();
        if data.len() != PUBLIC_KEY_LENGTH {
            return Err(NEARLedgerError::APDUExchangeError(format!(
                "`{}` response obtained of invalid length {} != {} (expected)",
                hex::encode(data),
                data.len(),
                PUBLIC_KEY_LENGTH
            )));
        }
        let mut bytes: [u8; PUBLIC_KEY_LENGTH] = [0u8; PUBLIC_KEY_LENGTH];
        bytes.copy_from_slice(data);

        let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|err| {
            NEARLedgerError::APDUExchangeError(format!(
                "problem constructing `ed25519_dalek::VerifyingKey` from \
                received byte array: {}, err: {:?}",
                hex::encode(data),
                err
            ))
        })?;
        Ok(key)
    } else {
        Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
            response.retcode(),
        )))
    }
}

fn get_transport() -> Result<TransportNativeHID, NEARLedgerError> {
    // instantiate the connection to Ledger
    // will return an error if Ledger is not connected
    let hidapi = HidApi::new().map_err(NEARLedgerError::HidApiError)?;
    TransportNativeHID::new(&hidapi).map_err(NEARLedgerError::LedgerHidError)
}

/// Sign the transaction. Transaction should be [borsh serialized](https://github.com/near/borsh-rs) `Vec<u8>`
///
/// # Inputs
/// * `unsigned_transaction_borsh_serializer` - unsigned transaction `near_primitives::transaction::Transaction`
///   which is serialized with `BorshSerializer` and basically is just `Vec<u8>`
/// * `seed_phrase_hd_path` - seed phrase hd path `near_slip10::BIP32Path` with which to sign
///
/// # Returns
///
/// * A `Result` whose `Ok` value is an `Signature` (bytes) and whose `Err` value is a
///   `NEARLedgerError` containing an error which occurred.
///
/// # Examples
///
/// ```no_run
/// use near_ledger::sign_transaction;
/// use near_primitives::{borsh, borsh::BorshSerialize};
/// use near_slip10::BIP32Path;
/// use std::str::FromStr;
///
/// # fn main() {
/// # let near_unsigned_transaction = [10; 250];
/// let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
/// let borsh_transaction = borsh::to_vec(&near_unsigned_transaction).unwrap();
/// let signature = sign_transaction(&borsh_transaction, hd_path).unwrap();
/// println!("{:#?}", signature);
/// # }
/// ```
///
/// # Trick
///
/// To convert the answer into `near_crypto::Signature` do:
///
/// ```
/// # let signature = [10; 64].to_vec();
/// let signature = near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
///     .expect("Signature is not expected to fail on deserialization");
/// ```
pub fn sign_transaction(
    unsigned_tx: BorshSerializedUnsignedTransaction,
    seed_phrase_hd_path: near_slip10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus(unsigned_tx, seed_phrase_hd_path, INS_SIGN_TRANSACTION)
}

#[derive(Debug, BorshSerialize)]
pub struct NEP413Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    pub callback_url: Option<String>,
}

pub fn sign_message_nep413(
    payload: &NEP413Payload,
    seed_phrase_hd_path: near_slip10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus(
        &borsh::to_vec(payload).unwrap(),
        seed_phrase_hd_path,
        INS_SIGN_NEP413_MESSAGE,
    )
}

pub fn sign_message_nep366_delegate_action(
    payload: BorshSerializedDelegateAction,
    seed_phrase_hd_path: near_slip10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus(
        payload,
        seed_phrase_hd_path,
        INS_SIGN_NEP366_DELEGATE_ACTION,
    )
}

/// this method should be kept in sync with [`crate::print_apdus::print_payload_internal`],
/// as avoiding copy-paste results in re-allocating `payload`
fn send_payload_apdus(
    payload: &[u8],
    seed_phrase_hd_path: near_slip10::BIP32Path,
    ins: u8,
) -> Result<SignatureBytes, NEARLedgerError> {
    let transport = get_transport()?;
    // seed_phrase_hd_path must be converted into bytes to be sent as `data` to the Ledger
    let hd_path_bytes = hd_path_to_bytes(&seed_phrase_hd_path);

    let mut data: Vec<u8> = vec![];
    data.extend(hd_path_bytes);
    data.extend_from_slice(payload);
    let chunks = data.chunks(CHUNK_SIZE);
    let chunks_count = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        let is_last_chunk = chunks_count == i + 1;
        let command = APDUCommand {
            cla: CLA,
            ins,
            p1: if is_last_chunk {
                P1_SIGN_NORMAL_LAST_CHUNK
            } else {
                P1_SIGN_NORMAL
            }, // Instruction parameter 1 (offset)
            p2: NETWORK_ID,
            data: chunk.to_vec(),
        };
        log_command(i, is_last_chunk, &command);
        match transport.exchange(&command) {
            Ok(response) => {
                log::info!(
                    "APDU out: {}\nAPDU ret code: {:x}",
                    hex::encode(response.apdu_data()),
                    response.retcode(),
                );
                // Ok means we successfully exchanged with the Ledger
                // but doesn't mean our request succeeded
                // we need to check it based on `response.retcode`
                if response.retcode() == RETURN_CODE_OK {
                    if is_last_chunk {
                        return Ok(response.data().to_vec());
                    }
                } else {
                    return Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                        response.retcode(),
                    )));
                }
            }
            Err(err) => return Err(NEARLedgerError::LedgerHIDError(err)),
        };
    }
    Err(NEARLedgerError::APDUExchangeError(
        "Unable to process request".to_owned(),
    ))
}
