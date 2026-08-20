#![no_std]

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::nfc::{Generic, i2c_nfc},
    metadata::{ModuleDetectResistor, ModuleID},
    registry::Registry,
};

pub struct NfcDriver;

impl DriverMeta for NfcDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R2K2,
        md1: ModuleDetectResistor::R2K4,
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

        let nfc = i2c_nfc::<Generic>(bus);
        registry.register(slot, NfcDriver::ID, nfc);

        Ok(())
    }
}
