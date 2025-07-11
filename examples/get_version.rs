use near_ledger::{get_version, NEARLedgerError};

fn main() -> Result<(), NEARLedgerError> {
    env_logger::builder().init();

    let version = get_version()?;

    log::info!("{version:#?}");
    Ok(())
}
