//! This library supports I2S communication for SPI version 1.2 (on STM32F1, STM32F2, STM32F4,
//! STM32L0, and STM32L1 microcontrollers).
//!
//! This library is normally used with a HAL library that provides a type that implements
//! [I2sPeripheral](crate::I2sPeripheral). An [I2sDriver](crate::I2sDriver) object can be created around the I2sPeripheral
//! object and used for I2S.

#![no_std]

extern crate nb;
extern crate vcell;

mod config;
pub mod format;
mod pac;

mod sealed {
    pub trait Sealed {}
}
//use self::sealed::Sealed;

use core::marker::PhantomData;

pub use self::config::{MasterClock, MasterConfig, SlaveConfig};
use self::pac::spi1::sr;
use self::pac::spi1::RegisterBlock;
//use crate::format::{DataFormat, FrameFormat, FrameSync};
//use crate::pac::spi1::i2scfgr::I2SCFG_A;

/// Marker, indicated master mode.
struct Master;

/// Marker, indicate slave mode.
struct Slave;

/// Clock polarity
#[derive(Debug, Clone)]
pub enum Polarity {
    /// Clock low when idle
    IdleLow,
    /// Clock high when idle
    IdleHigh,
}

/// The channel associated with a sample
#[derive(Debug, Clone, PartialEq)]
pub enum Channel {
    /// Left channel (word select low)
    Left,
    /// Right channel (word select high)
    Right,
}

/// Content of the status register.
pub struct Status {
    value: sr::R,
}

impl Status {
    /// Get the FRE flag. If `true` a frame error occured.
    ///
    /// This flag can be set by hardware only if the I2sDriver is configured in Slave mode. It is set
    /// when the WS line change at an unexpected moment. Usually, this indicate a synchronisation
    /// issue. This flag is cleared when reading the status register.
    pub fn fre(&self) -> bool {
        self.value.fre().bit()
    }

    /// Get the BSY flag. If `true` the I2s device is busy communicating.
    pub fn bsy(&self) -> bool {
        self.value.bsy().bit()
    }

    /// Get the OVR flag. If `true` an overrun error occured.
    ///
    /// This flag is set when data are received and the previous data have not yet been read. As a
    /// result, the incoming data are lost. This flag is cleared by a read operation on the data
    /// register followed by a read to the status register.
    pub fn ovr(&self) -> bool {
        self.value.ovr().bit()
    }

    /// Get the UDR flag. If `true` an underrun error occured.
    ///
    /// This flag can be set only in slave transmission mode. It is set when the first clock for
    /// data transmission appears while the software has not yet loaded any value into the data
    /// register.
    /// This flag is cleared by reading the status register.
    pub fn udr(&self) -> bool {
        self.value.udr().bit()
    }

    /// Get the CHSIDE flag. It indicate the channel has been received or to be transmitted. Have
    /// no meaning with PCM standard.
    ///
    /// This flag is updated when TXE or RXNE flags are set. This flag is meaningless and therefore
    /// not reliable is case of error or when using the PCM standard.
    pub fn chside(&self) -> Channel {
        match self.value.udr().bit() {
            false => Channel::Left,
            true => Channel::Right,
        }
    }

    /// Get the TXE flag. If `true` the Tx buffer is empty and the next data can be loaded into it.
    ///
    /// This flag can be set only in transmision mode. This flag is cleared by writing into the
    /// data register or by disabling the I2s peripheral.
    pub fn txe(&self) -> bool {
        self.value.txe().bit()
    }

    /// Get the RXNE flag. If `true` a valid received data is present in the Rx buffer.
    ///
    /// This flag can be only set in reception mode. It is cleared when the data register is read.
    pub fn rxne(&self) -> bool {
        self.value.rxne().bit()
    }
}

#[derive(Debug, Clone, Copy)]
enum SlaveOrMaster {
    Slave,
    Master,
}

#[derive(Debug, Clone, Copy)]
enum TransmitOrReceive {
    Transmit,
    Receive,
}

#[derive(Debug, Clone, Copy)]
/// I2s Configuration builder.
pub struct Config<MS> {
    slave_or_master: SlaveOrMaster,
    transmit_or_receive: TransmitOrReceive,

    _ms: PhantomData<MS>,
}

impl Config<Slave> {
    /// Create a new default slave configuration.
    pub fn new_slave() -> Self {
        Self {
            slave_or_master: SlaveOrMaster::Slave,
            transmit_or_receive: TransmitOrReceive::Transmit,
            _ms: PhantomData,
        }
    }
}

impl Config<Master> {
    /// Create a new default master configuration.
    pub fn new_master() -> Self {
        Self {
            slave_or_master: SlaveOrMaster::Master,
            transmit_or_receive: TransmitOrReceive::Transmit,
            _ms: PhantomData,
        }
    }
}

impl Default for Config<Slave> {
    /// Create a default configuration. It correspond to a default slave configuration.
    fn default() -> Self {
        Self::new_slave()
    }
}

impl<MS> Config<MS> {
    /// Instantiate the driver.
    pub fn i2s_driver<I: I2sPeripheral>(self, i2s_peripheral: I) -> I2sDriver<I> {
        let driver = I2sDriver { i2s_peripheral };
        driver.registers().cr1.reset(); // ensure SPI is disabled
        driver.registers().i2scfgr.write(|w| {
            w.i2smod().i2smode();
            match (self.slave_or_master, self.transmit_or_receive) {
                (SlaveOrMaster::Slave, TransmitOrReceive::Transmit) => w.i2scfg().slave_tx(),
                (SlaveOrMaster::Slave, TransmitOrReceive::Receive) => w.i2scfg().slave_rx(),
                (SlaveOrMaster::Master, TransmitOrReceive::Transmit) => w.i2scfg().master_tx(),
                (SlaveOrMaster::Master, TransmitOrReceive::Receive) => w.i2scfg().master_rx(),
            }
        });
        driver
    }
}

/// An object composed of a SPI device that can be used for I2S communication.
///
/// This trait is meant to be implemented on a type that represent a full SPI device, that means an
/// object composed of a SPI peripheral, pins used by it, and eventually a clock object (can be a
/// reference).
///
/// # Safety
///
/// It is only safe to implement this trait when:
///
/// * The implementing type has ownership of the peripheral, preventing any other accesses to the
///   register block.
/// * `REGISTERS` is a pointer to that peripheral's register block and can be safely accessed  as
///   long as ownership or a borrow of the implementing type is present.
pub unsafe trait I2sPeripheral {
    /// Pointer to the SPI register block
    const REGISTERS: *const ();
}

/// Driver of a SPI peripheral in I2S mode
///
/// # Example
///
/// TODO
///
/// ```no_run
/// ```
///
pub struct I2sDriver<I> {
    i2s_peripheral: I,
}

impl<I> I2sDriver<I>
where
    I: I2sPeripheral,
{
    /// Returns a reference to the register block
    fn registers(&self) -> &RegisterBlock {
        unsafe { &*(I::REGISTERS as *const RegisterBlock) }
    }

    /// Resets the values of all control and configuration registers
    fn reset_registers(&self) {
        let registers = self.registers();
        registers.cr1.reset();
        registers.cr2.reset();
        registers.i2scfgr.reset();
        registers.i2spr.reset();
    }
}

impl<I> I2sDriver<I>
where
    I: I2sPeripheral,
{
    /// Enable the I2S peripheral.
    pub fn enable(&mut self) {
        self.registers().i2scfgr.modify(|_, w| w.i2se().enabled());
    }

    /// Immediately Disable the I2S peripheral.
    ///
    /// It's up to the caller to not disable the peripheral in the middle of a frame.
    pub fn disable(&mut self) {
        self.registers().i2scfgr.modify(|_, w| w.i2se().disabled());
    }

    /// Get the content of the status register. It's content may modified during the operation.
    ///
    /// When reading the status register, the hardware may reset some error flag of it. The way
    /// each flag can be modified is documented on each [Status] flag getter.
    pub fn status(&mut self) -> Status {
        Status {
            value: self.registers().sr.read(),
        }
    }

    /// Write a raw half word to the Tx buffer and delete the TXE flag in status register.
    ///
    /// It's up to the caller to write the content when it's empty.
    pub fn write_data_register(&mut self, value: u16) {
        self.registers().dr.write(|w| w.dr().bits(value));
    }

    /// Read a raw value from the Rx buffer and delete the RXNE flag in status register.
    pub fn read_data_register(&mut self) -> u16 {
        self.registers().dr.read().dr().bits()
    }

    /// When set to `true`, an interrupt is generated each time the Tx buffer is empty.
    pub fn set_tx_interrupt(&mut self, enabled: bool) {
        self.registers().cr2.modify(|_, w| w.txeie().bit(enabled))
    }

    /// When set to `true`, an interrupt is generated each time the Rx buffer contains a new data.
    pub fn set_rx_interrupt(&mut self, enabled: bool) {
        self.registers().cr2.modify(|_, w| w.rxneie().bit(enabled))
    }

    /// When set to `true`, an interrupt is generated each time an error occurs.
    pub fn set_error_interrupt(&mut self, enabled: bool) {
        self.registers().cr2.modify(|_, w| w.errie().bit(enabled))
    }

    /// When set to `true`, a dma request is generated each time the Tx buffer is empty.
    pub fn set_tx_dma(&mut self, enabled: bool) {
        self.registers().cr2.modify(|_, w| w.txdmaen().bit(enabled))
    }

    /// When set to `true`, a dma request is generated each time the Rx buffer contains a new data.
    pub fn set_rx_dma(&mut self, enabled: bool) {
        self.registers().cr2.modify(|_, w| w.rxdmaen().bit(enabled))
    }

    /// Is the signal on WS line high ?
    pub fn ws_is_high(&self) -> bool {
        todo!()
    }

    /// Is the signal on WS line low ?
    pub fn ws_is_low(&self) -> bool {
        todo!()
    }

    //TODO method to get a handle to WS pin. It may usefull for setting an interrupt on pin to
    //synchronise I2s in slave mode
}
