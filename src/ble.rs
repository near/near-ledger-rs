//! BLE (Bluetooth Low Energy) transport for communicating with Ledger devices.
//!
//! This module provides [`TransportBle`], which connects to a BLE-capable Ledger device
//! (Nano X, Stax, Flex, or Nano Gen5) and exchanges APDU commands using the Ledger BLE
//! framing protocol.
//!
//! # BLE Framing Protocol
//!
//! Ledger BLE uses a packet-based protocol on top of GATT characteristics:
//! - **MTU negotiation**: Command tag `0x08`, retrieves the maximum payload per packet.
//! - **APDU exchange**: Command tag `0x05`, sends an APDU and reads the response.
//! - **Initial packet**: `[tag, seq_hi, seq_lo, len_hi, len_lo, ...payload]`
//! - **Subsequent packets**: `[tag, seq_hi, seq_lo, ...payload]`

use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter,
    ValueNotification, WriteType,
};
use btleplug::platform::{Manager, Peripheral};
use byteorder::{BigEndian, WriteBytesExt};
use futures::stream::BoxStream;
use futures::StreamExt;
use ledger_apdu::APDUAnswer;
use ledger_transport::APDUCommand;
use tokio::sync::Mutex;
use uuid::Uuid;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use std::ops::Deref;
use std::time::Duration;

use crate::NEARLedgerError;

// Ledger BLE service UUIDs
// https://developers.ledger.com/docs/device-interaction/references/identifiers

const NANO_X_SERVICE_UUID: Uuid = Uuid::from_fields(
    0x13d63400,
    0x2c97,
    0x0004,
    &[0x00, 0x00, 0x4c, 0x65, 0x64, 0x67, 0x65, 0x72],
);

const STAX_SERVICE_UUID: Uuid = Uuid::from_fields(
    0x13d63400,
    0x2c97,
    0x6004,
    &[0x00, 0x00, 0x4c, 0x65, 0x64, 0x67, 0x65, 0x72],
);

const FLEX_SERVICE_UUID: Uuid = Uuid::from_fields(
    0x13d63400,
    0x2c97,
    0x3004,
    &[0x00, 0x00, 0x4c, 0x65, 0x64, 0x67, 0x65, 0x72],
);

const NANO_GEN5_SERVICE_UUID: Uuid = Uuid::from_fields(
    0x13d63400,
    0x2c97,
    0x8004,
    &[0x00, 0x00, 0x4c, 0x65, 0x64, 0x67, 0x65, 0x72],
);

/// Nano Gen5 bootloader / early OS
const NANO_GEN5_BOOTLOADER_SERVICE_UUID: Uuid = Uuid::from_fields(
    0x13d63400,
    0x2c97,
    0x9004,
    &[0x00, 0x00, 0x4c, 0x65, 0x64, 0x67, 0x65, 0x72],
);

const LEDGER_SERVICE_UUIDS: &[Uuid] = &[
    NANO_X_SERVICE_UUID,
    STAX_SERVICE_UUID,
    FLEX_SERVICE_UUID,
    NANO_GEN5_SERVICE_UUID,
    NANO_GEN5_BOOTLOADER_SERVICE_UUID,
];

const MTU_COMMAND_TAG: u8 = 0x08;
const APDU_COMMAND_TAG: u8 = 0x05;
const BLE_SCAN_DURATION_SECS: u64 = 5;
const DEFAULT_MTU_SIZE: usize = 20;
const BLE_RESPONSE_TIMEOUT_SECS: u64 = 300;

fn is_ledger_service_uuid(uuid: &Uuid) -> bool {
    LEDGER_SERVICE_UUIDS.contains(uuid)
}

/// Derive the notify characteristic UUID from a Ledger service UUID.
///
/// Ledger characteristic UUIDs follow a pattern: service has `0000` in
/// d4 bytes 0-1, notify has `0001`, write has `0002`.
fn notify_uuid_for_service(service: &Uuid) -> Uuid {
    let (d1, d2, d3, d4) = service.as_fields();
    let mut d4_new = *d4;
    d4_new[1] = 0x01;
    Uuid::from_fields(d1, d2, d3, &d4_new)
}

fn write_uuid_for_service(service: &Uuid) -> Uuid {
    let (d1, d2, d3, d4) = service.as_fields();
    let mut d4_new = *d4;
    d4_new[1] = 0x02;
    Uuid::from_fields(d1, d2, d3, &d4_new)
}

/// BLE-specific errors
#[derive(Debug)]
pub enum BleError {
    Ble(btleplug::Error),
    DeviceNotFound,
    InvalidPacket,
    Comm(&'static str),
    Timeout,
}

impl fmt::Display for BleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BleError::Ble(err) => write!(f, "BLE error: {}", err),
            BleError::DeviceNotFound => write!(f, "No Ledger device found via BLE"),
            BleError::InvalidPacket => write!(f, "Invalid BLE packet received from Ledger"),
            BleError::Comm(msg) => write!(f, "BLE communication error: {}", msg),
            BleError::Timeout => write!(f, "Timeout waiting for Ledger BLE response"),
        }
    }
}

impl From<btleplug::Error> for BleError {
    fn from(err: btleplug::Error) -> Self {
        BleError::Ble(err)
    }
}

impl From<BleError> for NEARLedgerError {
    fn from(err: BleError) -> Self {
        NEARLedgerError::BleError(format!("{}", err))
    }
}

/// `[tag, seq(2), data_len(2), ...payload]`
#[derive(Debug)]
struct InitialPacket {
    command_tag: u8,
    data_length: u16,
    payload: Vec<u8>,
}

impl InitialPacket {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.payload.len() + 5);
        buf.write_u8(self.command_tag).unwrap();
        buf.write_u16::<BigEndian>(0).unwrap(); // sequence index
        buf.write_u16::<BigEndian>(self.data_length).unwrap();
        buf.extend(self.payload);
        buf
    }
}

impl TryFrom<Vec<u8>> for InitialPacket {
    type Error = BleError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 5 {
            return Err(BleError::InvalidPacket);
        }
        let payload = if value.len() == 5 {
            Vec::new()
        } else {
            value[5..].to_vec()
        };
        Ok(InitialPacket {
            command_tag: value[0],
            data_length: u16::from_be_bytes([value[3], value[4]]),
            payload,
        })
    }
}

/// `[tag, seq(2), ...payload]`
#[derive(Debug)]
struct SubsequentPacket {
    command_tag: u8,
    packet_sequence_index: u16,
    payload: Vec<u8>,
}

impl SubsequentPacket {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.payload.len() + 3);
        buf.write_u8(self.command_tag).unwrap();
        buf.write_u16::<BigEndian>(self.packet_sequence_index)
            .unwrap();
        buf.extend(self.payload);
        buf
    }
}

impl TryFrom<Vec<u8>> for SubsequentPacket {
    type Error = BleError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(BleError::InvalidPacket);
        }
        let payload = if value.len() == 3 {
            Vec::new()
        } else {
            value[3..].to_vec()
        };
        Ok(SubsequentPacket {
            command_tag: value[0],
            packet_sequence_index: u16::from_be_bytes([value[1], value[2]]),
            payload,
        })
    }
}

/// BLE transport for communicating with a Ledger device.
///
/// # Example
///
/// ```no_run
/// use near_ledger::{TransportBle, BleError};
///
/// # async fn example() -> Result<(), BleError> {
/// let transport = TransportBle::new().await?;
/// // Use with BLE API functions...
/// # Ok(())
/// # }
/// ```
pub struct TransportBle {
    device: Peripheral,
    write_characteristic: Characteristic,
    notifications: Mutex<BoxStream<'static, ValueNotification>>,
    mtu_size: Mutex<Option<usize>>,
}

impl fmt::Debug for TransportBle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransportBle")
            .field("write_characteristic", &self.write_characteristic)
            .finish()
    }
}

impl TransportBle {
    async fn is_ledger(peripheral: &Peripheral) -> bool {
        if let Ok(Some(props)) = peripheral.properties().await {
            if let Some(ref name) = props.local_name {
                if name.starts_with("Nano X")
                    || name.starts_with("Nano S Plus")
                    || name.starts_with("Stax")
                    || name.starts_with("Flex")
                    || name.starts_with("Ledger")
                {
                    return true;
                }
            }
            for svc in &props.services {
                if is_ledger_service_uuid(svc) {
                    return true;
                }
            }
        }
        false
    }

    /// Scan for available Ledger devices via BLE.
    pub async fn list_ledgers() -> Result<Vec<Peripheral>, BleError> {
        let manager = Manager::new().await?;
        let adapter_list = manager.adapters().await?;

        let mut ledgers = Vec::new();

        for adapter in adapter_list.iter() {
            adapter.start_scan(ScanFilter::default()).await?;
            tokio::time::sleep(Duration::from_secs(BLE_SCAN_DURATION_SECS)).await;
            adapter.stop_scan().await?;

            let peripherals = adapter.peripherals().await?;
            for peripheral in peripherals {
                if Self::is_ledger(&peripheral).await {
                    ledgers.push(peripheral);
                }
            }
        }

        Ok(ledgers)
    }

    /// Scan for and connect to the first available Ledger device.
    pub async fn new() -> Result<Self, BleError> {
        let ledgers = Self::list_ledgers().await?;
        let ledger = ledgers.into_iter().next().ok_or(BleError::DeviceNotFound)?;
        Self::connect(ledger).await
    }

    /// Connect to a specific BLE peripheral as a Ledger device.
    ///
    /// The peripheral should be one returned by [`list_ledgers`](Self::list_ledgers).
    pub async fn connect(peripheral: Peripheral) -> Result<Self, BleError> {
        if !peripheral.is_connected().await? {
            peripheral.connect().await?;
        }
        peripheral.discover_services().await?;

        let characteristics = peripheral.characteristics();

        let service_uuid = characteristics
            .iter()
            .map(|c| c.service_uuid)
            .find(is_ledger_service_uuid);

        let (write_characteristic, notify_characteristic) = if let Some(svc) = service_uuid {
            let expected_write = write_uuid_for_service(&svc);
            let expected_notify = notify_uuid_for_service(&svc);

            let write_char = characteristics
                .iter()
                .find(|c| c.uuid == expected_write)
                .cloned()
                .ok_or(BleError::Comm(
                    "Ledger WRITE characteristic not found for service",
                ))?;

            let notify_char = characteristics
                .iter()
                .find(|c| c.uuid == expected_notify)
                .cloned()
                .ok_or(BleError::Comm(
                    "Ledger NOTIFY characteristic not found for service",
                ))?;

            (write_char, notify_char)
        } else {
            // Fallback for unknown future devices: match by GATT property flags
            let write_char = characteristics
                .iter()
                .find(|c| c.properties.contains(CharPropFlags::WRITE))
                .cloned()
                .ok_or(BleError::Comm(
                    "No WRITE characteristic found on Ledger device",
                ))?;

            let notify_char = characteristics
                .iter()
                .find(|c| c.properties.contains(CharPropFlags::NOTIFY))
                .cloned()
                .ok_or(BleError::Comm(
                    "No NOTIFY characteristic found on Ledger device",
                ))?;

            (write_char, notify_char)
        };

        peripheral.subscribe(&notify_characteristic).await?;
        let notifications = peripheral.notifications().await?;

        let transport = TransportBle {
            device: peripheral,
            write_characteristic,
            notifications: Mutex::new(notifications),
            mtu_size: Mutex::new(None),
        };

        transport.infer_mtu().await?;

        Ok(transport)
    }

    async fn write(&self, data: &[u8]) -> Result<(), BleError> {
        log::info!("BLE => {}", hex::encode(data));
        self.device
            .write(&self.write_characteristic, data, WriteType::WithResponse)
            .await?;
        Ok(())
    }

    async fn next_notification(&self) -> Result<Vec<u8>, BleError> {
        let timeout = Duration::from_secs(BLE_RESPONSE_TIMEOUT_SECS);
        let mut stream = self.notifications.lock().await;
        match tokio::time::timeout(timeout, stream.next()).await {
            Ok(Some(notif)) => {
                log::info!("BLE <= {}", hex::encode(&notif.value));
                Ok(notif.value)
            }
            Ok(None) => Err(BleError::Comm("BLE notification stream ended unexpectedly")),
            Err(_) => Err(BleError::Timeout),
        }
    }

    /// Send a framed request, splitting into BLE packets as needed.
    async fn send_framed(
        &self,
        command_tag: u8,
        payload: &[u8],
        mtu_size: usize,
    ) -> Result<(), BleError> {
        let first_chunk_data_size = mtu_size.saturating_sub(5);
        let subsequent_chunk_data_size = mtu_size.saturating_sub(3);

        if payload.len() > u16::MAX as usize {
            return Err(BleError::Comm(
                "Payload too large for BLE framing (>65535 bytes)",
            ));
        }

        let first_data_end = std::cmp::min(first_chunk_data_size, payload.len());
        let initial_packet = InitialPacket {
            command_tag,
            data_length: payload.len() as u16,
            payload: payload[..first_data_end].to_vec(),
        };
        self.write(&initial_packet.serialize()).await?;

        let mut offset = first_data_end;
        let mut seq_index: u16 = 1;
        while offset < payload.len() {
            let end = std::cmp::min(offset + subsequent_chunk_data_size, payload.len());
            let packet = SubsequentPacket {
                command_tag,
                packet_sequence_index: seq_index,
                payload: payload[offset..end].to_vec(),
            };
            self.write(&packet.serialize()).await?;
            offset = end;
            seq_index += 1;
        }

        Ok(())
    }

    /// Receive a framed response, reassembling from BLE packets.
    async fn recv_framed(&self, command_tag: u8) -> Result<Vec<u8>, BleError> {
        let raw = loop {
            let data = self.next_notification().await?;
            if data.is_empty() {
                return Err(BleError::InvalidPacket);
            }
            if data[0] == command_tag {
                break data;
            }
            log::info!("BLE: skipping notification with tag 0x{:02x}", data[0]);
        };

        let initial_packet: InitialPacket = raw.try_into()?;
        log::info!(
            "BLE recv initial: tag=0x{:02x} data_length={} payload_len={}",
            initial_packet.command_tag,
            initial_packet.data_length,
            initial_packet.payload.len()
        );

        let total_length = initial_packet.data_length as usize;
        let mut buffer = initial_packet.payload;

        while buffer.len() < total_length {
            let raw = self.next_notification().await?;
            let packet: SubsequentPacket = raw.try_into()?;
            if packet.command_tag != command_tag {
                log::info!(
                    "BLE: skipping subsequent packet with tag 0x{:02x}",
                    packet.command_tag
                );
                continue;
            }
            log::info!(
                "BLE recv subsequent: seq={} payload_len={}",
                packet.packet_sequence_index,
                packet.payload.len()
            );
            buffer.extend(packet.payload);
        }

        buffer.truncate(total_length);
        Ok(buffer)
    }

    /// Negotiate the MTU with the Ledger device.
    ///
    /// The MTU response does NOT follow the standard framing. Per the official
    /// Ledger JS reference (`TransportWebBLE.inferMTU`), byte at index 5 of the
    /// raw notification is the mtu_size value used by `send_framed`.
    async fn infer_mtu(&self) -> Result<usize, BleError> {
        self.write(&[MTU_COMMAND_TAG, 0x00, 0x00, 0x00, 0x00])
            .await?;

        // Read raw response (not using recv_framed — non-standard format)
        let raw = loop {
            let data = self.next_notification().await?;
            if !data.is_empty() && data[0] == MTU_COMMAND_TAG {
                break data;
            }
            log::info!(
                "BLE: skipping non-MTU notification with tag 0x{:02x}",
                data.first().copied().unwrap_or(0)
            );
        };

        if raw.len() < 6 {
            log::warn!(
                "BLE: MTU response too short ({} bytes), using default {}",
                raw.len(),
                DEFAULT_MTU_SIZE
            );
            let mut mtu = self.mtu_size.lock().await;
            *mtu = Some(DEFAULT_MTU_SIZE);
            return Ok(DEFAULT_MTU_SIZE);
        }

        let mtu_size = raw[5] as usize;
        // Initial packet header is 5 bytes, so mtu_size must be > 5 to carry any data.
        // Fall back to default if the device returns something unusably small.
        let mtu_size = if mtu_size > 5 {
            mtu_size
        } else {
            log::warn!(
                "BLE: negotiated mtu_size={} too small, using default {}",
                mtu_size,
                DEFAULT_MTU_SIZE
            );
            DEFAULT_MTU_SIZE
        };
        log::info!("BLE: mtu_size={}", mtu_size);

        let mut mtu = self.mtu_size.lock().await;
        *mtu = Some(mtu_size);
        Ok(mtu_size)
    }

    async fn get_mtu_size(&self) -> usize {
        let mtu = self.mtu_size.lock().await;
        mtu.unwrap_or(DEFAULT_MTU_SIZE)
    }

    /// Exchange an APDU command with the Ledger device over BLE.
    pub async fn exchange<I: Deref<Target = [u8]>>(
        &self,
        command: &APDUCommand<I>,
    ) -> Result<APDUAnswer<Vec<u8>>, BleError> {
        let apdu_data = command.serialize();
        let mtu_size = self.get_mtu_size().await;

        self.send_framed(APDU_COMMAND_TAG, &apdu_data, mtu_size)
            .await?;

        let answer = self.recv_framed(APDU_COMMAND_TAG).await?;
        APDUAnswer::from_answer(answer).map_err(|_| BleError::Comm("Response was too short"))
    }
}
