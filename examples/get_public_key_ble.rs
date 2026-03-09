use std::str::FromStr;

use near_ledger::{get_public_key_ble, open_near_application_ble, NEARLedgerError, TransportBle};
use slipped10::BIP32Path;

#[path = "common/lib.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), NEARLedgerError> {
    env_logger::builder().init();

    log::info!("Scanning for Ledger devices via BLE...");
    let transport = TransportBle::new()
        .await
        .map_err(|e| NEARLedgerError::BleError(format!("{}", e)))?;
    log::info!("Connected to Ledger via BLE");

    open_near_application_ble(&transport).await?;
    let hd_path = BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap();
    let public_key = get_public_key_ble(&transport, hd_path).await?;

    common::display_pub_key(public_key);

    Ok(())
}
