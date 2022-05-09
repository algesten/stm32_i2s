//! Abstraction for I2S transfer
//!
//!

use crate::Config as DriverConfig;
use crate::I2sDriver as Driver;
use crate::*;

#[derive(Debug, Clone, Copy)]
/// I2s TransferConfiguration builder.
///
///  - `MS`: `Master` or `Slave`
///  - `TR`: `Transmit` or `Receive`
pub struct TransferConfig<MS, TR> {
    driver_config: DriverConfig<MS, TR>,
}

impl TransferConfig<Slave, Transmit> {
    /// Create a new default slave configuration.
    pub fn new_slave() -> Self {
        Self {
            driver_config: DriverConfig::new_slave(),
        }
    }
}

impl TransferConfig<Master, Transmit> {
    /// Create a new default master configuration.
    pub fn new_master() -> Self {
        Self {
            driver_config: DriverConfig::new_master(),
        }
    }
}

impl<MS, TR> TransferConfig<MS, TR> {
    /// Create a `Transfer` object.
    pub fn i2s_transfer<I: I2sPeripheral>(self, i2s_peripheral: I) -> Transfer<I, Mode<MS, TR>> {
        let driver = self.driver_config.i2s_driver(i2s_peripheral);
        Transfer::<I, Mode<MS, TR>> { driver }
    }
}

impl Default for TransferConfig<Slave, Transmit> {
    /// Create a default configuration. It correspond to a default slave configuration.
    fn default() -> Self {
        Self::new_slave()
    }
}

impl<MS, TR> TransferConfig<MS, TR> {
    /// Configure transfert for transmission.
    pub fn transmit(self) -> TransferConfig<MS, Transmit> {
        TransferConfig::<MS, Transmit> {
            driver_config: self.driver_config.transmit(),
        }
    }
    /// TransferConfigure in transmit mode
    pub fn receive(self) -> TransferConfig<MS, Receive> {
        TransferConfig::<MS, Receive> {
            driver_config: self.driver_config.receive(),
        }
    }
    /// Select the I2s standard to use
    pub fn standard(self, standard: I2sStandard) -> Self {
        TransferConfig::<MS, TR> {
            driver_config: self.driver_config.standard(standard),
        }
    }
    /// Select steady state clock polarity
    pub fn clock_polarity(self, polarity: ClockPolarity) -> Self {
        TransferConfig::<MS, TR> {
            driver_config: self.driver_config.clock_polarity(polarity),
        }
    }

    /// Select data format
    pub fn data_format(self, format: DataFormat) -> Self {
        TransferConfig::<MS, TR> {
            driver_config: self.driver_config.data_format(format),
        }
    }

    /// Convert to a slave configuration. This delete Master Only Settings.
    pub fn to_slave(self) -> TransferConfig<Slave, TR> {
        TransferConfig::<Slave, TR> {
            driver_config: self.driver_config.to_slave(),
        }
    }

    /// Convert to a master configuration.
    pub fn to_master(self) -> TransferConfig<Master, TR> {
        TransferConfig::<Master, TR> {
            driver_config: self.driver_config.to_master(),
        }
    }
}

impl<TR> TransferConfig<Master, TR> {
    /// Enable/Disable Master Clock. Affect the effective sampling rate.
    ///
    /// This can be only set and only have meaning for Master mode.
    pub fn master_clock(self, enable: bool) -> Self {
        TransferConfig::<Master, TR> {
            driver_config: self.driver_config.master_clock(enable),
        }
    }

    /// TransferConfigure audio frequency by setting the prescaler with an odd factor and a divider.
    ///
    /// The effective sampling frequency is:
    ///  - `i2s_clock / [256 * ((2 * div) + odd)]` when master clock is enabled
    ///  - `i2s_clock / [(channel_length * 2) * ((2 * div) + odd)]` when master clock is disabled
    ///
    ///  `i2s_clock` is I2S clock source frequency, and `channel_length` is width in bits of the
    ///  channel (see [DataFormat])
    ///
    /// This setting only have meaning and can be only set for master.
    ///
    /// # Panics
    ///
    /// `div` must be at least 2, otherwise the method panics.
    pub fn prescaler(self, odd: bool, div: u8) -> Self {
        TransferConfig::<Master, TR> {
            driver_config: self.driver_config.prescaler(odd, div),
        }
    }

    /// Request an audio sampling frequency. The effective audio sampling frequency may differ.
    pub fn request_frequency(self, freq: u32) -> Self {
        TransferConfig::<Master, TR> {
            driver_config: self.driver_config.request_frequency(freq),
        }
    }

    /// Require exactly this audio sampling frequency.
    ///
    /// If the required frequency can not bet set, Instantiate the driver will produce a error
    pub fn require_frequency(self, freq: u32) -> Self {
        TransferConfig::<Master, TR> {
            driver_config: self.driver_config.require_frequency(freq),
        }
    }
}

pub struct Transfer<I: I2sPeripheral, MODE> {
    driver: Driver<I, MODE>,
}
