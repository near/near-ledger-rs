use near_ledger::{get_version_ble, open_near_application_ble, NEARLedgerError, TransportBle};

#[tokio::main]
async fn main() -> Result<(), NEARLedgerError> {
    env_logger::builder().init();

    log::info!("Scanning for Ledger devices via BLE...");
    let transport = TransportBle::new()
        .await
        .map_err(|e| NEARLedgerError::BleError(format!("{}", e)))?;
    log::info!("Connected to Ledger via BLE");

    open_near_application_ble(&transport).await?;
    let version = get_version_ble(&transport).await?;
    log::info!("NEAR app version: {:#?}", version);

    Ok(())
}
