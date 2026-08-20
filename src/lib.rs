#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use xpanse_api::{
    bus::{
        allocator::BusAllocator,
        i2c::I2cBusHandle,
    },
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::nfc::{Nfc, NfcError},
    metadata::{ModuleDetectResistor, ModuleID},
    registry::Registry,
};

/// I2C-based NFC device (e.g. ST25DV04K).
struct I2cNfc {
    bus: I2cBusHandle,
}

impl I2cNfc {
    fn new(bus: I2cBusHandle) -> Self {
        Self { bus }
    }
}

impl Nfc for I2cNfc {
    fn read<'a>(
        &'a mut self,
        address: u16,
        data: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), NfcError>> + 'a>> {
        Box::pin(async move {
            let addr_bytes = address.to_be_bytes();
            self.bus
                .write_read(0xA0, &addr_bytes, data)
                .await
                .map_err(|_| NfcError::BusError)
        })
    }

    fn write<'a>(
        &'a mut self,
        address: u16,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), NfcError>> + 'a>> {
        Box::pin(async move {
            if data.len() > 256 {
                return Err(NfcError::InvalidAddress);
            }
            let mut buf = [0u8; 258];
            let addr_bytes = address.to_be_bytes();
            buf[0..2].copy_from_slice(&addr_bytes);
            buf[2..2 + data.len()].copy_from_slice(data);
            self.bus
                .write(0xA0, &buf[0..2 + data.len()])
                .await
                .map_err(|_| NfcError::BusError)
        })
    }

    fn detect_field<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<bool, NfcError>> + 'a>> {
        Box::pin(async move {
            // Field detection register at address 0x0016
            let mut buf = [0u8; 1];
            self.read(0x0016, &mut buf).await?;
            Ok((buf[0] & 0x01) != 0)
        })
    }
}

pub struct NfcDriver;

impl DriverMeta for NfcDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K,
        md1: ModuleDetectResistor::R16K,
    };
}

impl<G: BankPins> Driver<G> for NfcDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: xpanse_api::metadata::ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        let bus = bus_allocator
            .create_i2c_bitbang(
                gpio_bank.gpio0.into(),
                gpio_bank.gpio1.into(),
                100_000,
            )
            .map_err(|_| DriverError::InitFailed)?;

        let nfc = Box::new(I2cNfc::new(bus)) as Box<dyn Nfc>;
        registry.register(slot, NfcDriver::ID, nfc);

        Ok(())
    }
}
