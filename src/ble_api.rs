//! Async BLE API functions mirroring the USB HID API.
//!
//! All functions take a [`TransportBle`] reference, which should be reused across calls.
//!
//! # Example
//!
//! ```no_run
//! use near_ledger::{TransportBle, get_version_ble};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let transport = TransportBle::new().await.map_err(|e| format!("{:?}", e))?;
//! let version = get_version_ble(&transport).await.map_err(|e| format!("{:?}", e))?;
//! println!("NEAR app version: {:?}", version);
//! # Ok(())
//! # }
//! ```

use ed25519_dalek::PUBLIC_KEY_LENGTH;
use ledger_transport::APDUCommand;
use std::time::Duration;
use tokio::time::sleep;

use crate::ble::TransportBle;
use crate::{
    hd_path_to_bytes, log_command, retcode_to_error_string, BorshSerializedDelegateAction,
    BorshSerializedUnsignedTransaction, NEARLedgerAppVersion, NEARLedgerError, NEP413Payload,
    SignatureBytes, CHUNK_SIZE, CLA, CLA_DASHBOARD, CLA_OPEN_APP, INS_GET_APP_NAME,
    INS_GET_PUBLIC_KEY, INS_GET_VERSION, INS_GET_WALLET_ID, INS_OPEN_APP, INS_QUIT_APP,
    INS_SIGN_NEP366_DELEGATE_ACTION, INS_SIGN_NEP413_MESSAGE, INS_SIGN_TRANSACTION, NETWORK_ID,
    P1_GET_PUB_DISPLAY, P1_GET_PUB_SILENT, P1_SIGN_NORMAL, P1_SIGN_NORMAL_LAST_CHUNK,
    RETURN_CODE_OK, RETURN_CODE_UNKNOWN_ERROR,
};

/// Get the version of the NEAR App installed on a BLE-connected Ledger device
pub async fn get_version_ble(
    transport: &TransportBle,
) -> Result<NEARLedgerAppVersion, NEARLedgerError> {
    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_VERSION,
        p1: 0,
        p2: 0,
        data: vec![],
    };

    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => {
            log::info!(
                "APDU out (BLE): {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );
            if response.retcode() == RETURN_CODE_OK {
                Ok(response.data().to_vec())
            } else {
                Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    response.retcode(),
                )))
            }
        }
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

async fn running_app_name_ble(transport: &TransportBle) -> Result<String, NEARLedgerError> {
    let command = APDUCommand {
        cla: CLA_DASHBOARD,
        ins: INS_GET_APP_NAME,
        p1: 0,
        p2: 0,
        data: vec![],
    };

    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => {
            log::info!(
                "APDU out (BLE): {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );
            match response.retcode() {
                RETURN_CODE_OK => {
                    let data = response.data();
                    if data.len() < 2 {
                        return Err(NEARLedgerError::APDUExchangeError(
                            "App name response too short".to_string(),
                        ));
                    }
                    let app_name_len = data[1] as usize;
                    if data.len() < 2 + app_name_len {
                        return Err(NEARLedgerError::APDUExchangeError(
                            "App name response truncated".to_string(),
                        ));
                    }
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
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

async fn quit_open_application_ble(transport: &TransportBle) -> Result<(), NEARLedgerError> {
    let command = APDUCommand {
        cla: CLA_DASHBOARD,
        ins: INS_QUIT_APP,
        p1: 0,
        p2: 0,
        data: vec![],
    };

    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => {
            log::info!(
                "APDU out (BLE): {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );
            match response.retcode() {
                RETURN_CODE_OK => Ok(()),
                retcode => Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    retcode,
                ))),
            }
        }
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

/// Open the NEAR application on a BLE-connected Ledger device.
/// No-op if NEAR app is already open.
pub async fn open_near_application_ble(transport: &TransportBle) -> Result<(), NEARLedgerError> {
    match running_app_name_ble(transport).await?.as_str() {
        "NEAR" => return Ok(()),
        "BOLOS" => {}
        _ => {
            quit_open_application_ble(transport).await?;
            // Wait for the Ledger to close the app
            sleep(Duration::from_secs(1)).await;
        }
    }

    let data = b"NEAR".to_vec();
    let command: APDUCommand<Vec<u8>> = APDUCommand {
        cla: CLA_OPEN_APP,
        ins: INS_OPEN_APP,
        p1: 0x00,
        p2: 0x00,
        data,
    };

    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => {
            log::info!("APDU ret code (BLE): {:x}", response.retcode());
            match response.retcode() {
                RETURN_CODE_OK => Ok(()),
                retcode => Err(NEARLedgerError::APDUExchangeError(retcode_to_error_string(
                    retcode,
                ))),
            }
        }
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

/// Gets PublicKey from a BLE-connected Ledger on the given `hd_path`.
/// The key will be displayed on the Ledger screen for user confirmation.
pub async fn get_public_key_ble(
    transport: &TransportBle,
    hd_path: slipped10::BIP32Path,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    get_public_key_with_display_flag_ble(transport, hd_path, true).await
}

/// Gets PublicKey from a BLE-connected Ledger, optionally displaying on the device screen
pub async fn get_public_key_with_display_flag_ble(
    transport: &TransportBle,
    hd_path: slipped10::BIP32Path,
    display_and_confirm: bool,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    let hd_path_bytes = hd_path_to_bytes(&hd_path);

    let p1 = if display_and_confirm {
        P1_GET_PUB_DISPLAY
    } else {
        P1_GET_PUB_SILENT
    };

    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_PUBLIC_KEY,
        p1,
        p2: NETWORK_ID,
        data: hd_path_bytes,
    };
    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => handle_public_key_response_ble(response),
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

/// Gets the Wallet ID from a BLE-connected Ledger on the given `hd_path`
pub async fn get_wallet_id_ble(
    transport: &TransportBle,
    hd_path: slipped10::BIP32Path,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    let hd_path_bytes = hd_path_to_bytes(&hd_path);

    let command = APDUCommand {
        cla: CLA,
        ins: INS_GET_WALLET_ID,
        p1: 0,
        p2: NETWORK_ID,
        data: hd_path_bytes,
    };
    log::info!("APDU  in (BLE): {}", hex::encode(command.serialize()));

    match transport.exchange(&command).await {
        Ok(response) => handle_public_key_response_ble(response),
        Err(err) => Err(NEARLedgerError::from(err)),
    }
}

fn handle_public_key_response_ble(
    response: ledger_apdu::APDUAnswer<Vec<u8>>,
) -> Result<ed25519_dalek::VerifyingKey, NEARLedgerError> {
    log::info!(
        "APDU out (BLE): {}\nAPDU ret code: {:x}",
        hex::encode(response.apdu_data()),
        response.retcode(),
    );
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

/// Sign a borsh-serialized transaction on a BLE-connected Ledger device
pub async fn sign_transaction_ble(
    transport: &TransportBle,
    unsigned_tx: BorshSerializedUnsignedTransaction<'_>,
    seed_phrase_hd_path: slipped10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus_ble(
        transport,
        unsigned_tx,
        seed_phrase_hd_path,
        INS_SIGN_TRANSACTION,
    )
    .await
}

/// Sign an NEP-413 off-chain message on a BLE-connected Ledger device
pub async fn sign_message_nep413_ble(
    transport: &TransportBle,
    payload: &NEP413Payload,
    seed_phrase_hd_path: slipped10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus_ble(
        transport,
        &borsh::to_vec(payload).unwrap(),
        seed_phrase_hd_path,
        INS_SIGN_NEP413_MESSAGE,
    )
    .await
}

/// Sign an NEP-366 delegate action on a BLE-connected Ledger device
pub async fn sign_message_nep366_delegate_action_ble(
    transport: &TransportBle,
    payload: BorshSerializedDelegateAction<'_>,
    seed_phrase_hd_path: slipped10::BIP32Path,
) -> Result<SignatureBytes, NEARLedgerError> {
    send_payload_apdus_ble(
        transport,
        payload,
        seed_phrase_hd_path,
        INS_SIGN_NEP366_DELEGATE_ACTION,
    )
    .await
}

async fn send_payload_apdus_ble(
    transport: &TransportBle,
    payload: &[u8],
    seed_phrase_hd_path: slipped10::BIP32Path,
    ins: u8,
) -> Result<SignatureBytes, NEARLedgerError> {
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
            },
            p2: NETWORK_ID,
            data: chunk.to_vec(),
        };
        log_command(i, is_last_chunk, &command);
        match transport.exchange(&command).await {
            Ok(response) => {
                log::info!(
                    "APDU out (BLE): {}\nAPDU ret code: {:x}",
                    hex::encode(response.apdu_data()),
                    response.retcode(),
                );
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
            Err(err) => return Err(NEARLedgerError::from(err)),
        };
    }
    Err(NEARLedgerError::APDUExchangeError(
        "Unable to process request".to_owned(),
    ))
}
