//! # Individual GPIO pins
//!
//! This crate fully implements the [digital::v2 traits of embedded_hal](https://docs.rs/embedded-hal/latest/embedded_hal/digital/v2/index.html).
//!
//! Due to the I2C overhead, this module offers two options for state management:
//! * [Regular access mode](RegularAccessMode): The state is synchronously updated when calling
//! state functions like `is_high()`, causing 1:1 I2C operations for each individual call.
//! * [Refresh access mode](RefreshMode): Register states are internally cached. Functions like
//! `is_high()` are just using the cached state. The state is updated explicitly, but for all pins at once.
//! In the best case, the I2C overhead is reduced to one eighth. See [below examples](#refreshable-access-mode) for more details.
//!
//! ## Setup
//! Individual pins can be fetched using [PCA9539](crate::expander::PCA9539) instance.
//! Different concurrency models are supported, see [Concurrency](#Concurrency) section for more details.
//! ```
//! use pca9539::example::DummyI2CBus;
//! use pca9539::expander::Bank::Bank0;
//! use pca9539::expander::PCA9539;
//! use pca9539::expander::PinID::Pin1;
//!
//! let i2c_bus = DummyI2CBus::default();
//! let mut  expander = PCA9539::new(i2c_bus, 0x74);
//! let pins = expander.pins();
//! ```
//! ## State management modes
//! ### Regular access mode
//! The following examples demonstrate using the synchronous regular access mode.
//! Regular access mode is used when calling `get_pin()` method.
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::Bank::{Bank0, Bank1};
//!# use pca9539::expander::PCA9539;
//!# use pca9539::expander::PinID::{Pin1, Pin2, Pin4};
//!# use embedded_hal::digital::v2::{InputPin, IoPin, PinState, OutputPin};
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//! let pins = expander.pins();
//! let pin12 = pins.get_pin(Bank1, Pin2);
//! let mut  pin04 = pins.get_pin(Bank0, Pin4).into_output_pin(PinState::Low).unwrap();
//!
//! // Fetching input state of Pin12
//! let is_high = pin12.is_high().unwrap();
//!
//! // Setting Pin04 to high output state
//! pin04.set_high().unwrap()
//! ```
//! ### Refreshable access mode
//! The following examples demonstrate using the refreshable access mode.
//! Regular access mode is used when calling `get_refreshable_pin()` method.
//!
//! In contrast to the previous method, the state must be explicitly updated/refreshed here.
//! It does not matter which pin is used to call update/refresh.
//! The state is always updated for all pins or pins of the same bank.
//!
//! As `is_high()` and `is_low()` are just acting on cached state, calls of this method can not fail.
//! #### Input example
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::Bank::{Bank0, Bank1};
//!# use pca9539::expander::PCA9539;
//!# use pca9539::expander::PinID::{Pin0, Pin1, Pin2, Pin3, Pin4};
//!# use embedded_hal::digital::v2::{InputPin, IoPin, PinState, OutputPin};
//!# use pca9539::pins::RefreshableInputPin;
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//! let pins = expander.pins();
//! let pin00 = pins.get_refreshable_pin(Bank0, Pin0);
//! let pin10 = pins.get_refreshable_pin(Bank1, Pin0);
//! let pin11 = pins.get_refreshable_pin(Bank1, Pin1);
//!
//! // Updates the input state of just Bank1. So input state of Pin10 and Pin11 is now up2date
//! pin10.refresh_bank().unwrap();
//! assert!(pin10.is_high().unwrap());
//! assert!(pin11.is_low().unwrap());
//!
//! // Updates the input state of all banks. So all pins are now up2date
//! pin00.refresh_bank().unwrap();
//! assert!(pin00.is_low().unwrap());
//! ```
//! #### Output example
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::Bank::{Bank0, Bank1};
//!# use pca9539::expander::PCA9539;
//!# use pca9539::expander::PinID::{Pin0, Pin1, Pin2, Pin3, Pin4};
//!# use embedded_hal::digital::v2::{InputPin, IoPin, PinState, OutputPin};
//!# use pca9539::pins::RefreshableOutputPin;
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//! let pins = expander.pins();
//! let mut pin00 = pins.get_refreshable_pin(Bank0, Pin0).into_output_pin(PinState::Low).unwrap();
//! let mut pin10 = pins.get_refreshable_pin(Bank1, Pin0).into_output_pin(PinState::Low).unwrap();
//! let mut pin11 = pins.get_refreshable_pin(Bank1, Pin1).into_output_pin(PinState::Low).unwrap();
//!
//! pin00.set_low().unwrap();
//! pin10.set_high().unwrap();
//! pin11.set_state(PinState::High).unwrap();
//!
//! // Writes the output state of just Bank1.
//! pin10.update_bank().unwrap();
//!
//! // Writes the output state of all banks.
//! pin00.update_all().unwrap();
//! ```
//!
//! ## Concurrency
//! As the pins are using a shared reference, some kind of concurrency management is required.
//! This crate currently offers three different concurrency guards. Which one should be used, depends
//! on the application type:
//!
//! ### Lock-free
//! Returns a pins container without using any locks
//! This is the most efficient way of using individual pins
//! The downside is, that these pins are neither Send or Sync, so can only be used in single-threaded
//! and interrupt-free applications
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::PCA9539;
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//! let pins = expander.pins();
//! ```
//!
//! ### CS Mutex (Cortex-M)
//! Returns a pins container using Mutex based on critical sections
//! Individual pins can be used across threads and interrupts, as long just running on a single core
//!
//! *Requires activation of `cortex-m` feature*
//!
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::PCA9539;
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//!# #[cfg(feature = "cortex-m")]
//! let pins = expander.pins_cs_mutex();
//! ```
//!
//! ### Spin Mutex
//! Returns a pins container using a spin mutex
//! This is safe to use across threads and on multi-core applications
//! However, this requires a system supporting spin mutexes, which are generally only
//! available on systems with Atomic CAS
//!
//! *Requires activation of `spin` feature*
//!
//! ```
//!# use pca9539::example::DummyI2CBus;
//!# use pca9539::expander::PCA9539;
//!#
//!# let i2c_bus = DummyI2CBus::default();
//!# let mut  expander = PCA9539::new(i2c_bus, 0x74);
//!# #[cfg(feature = "spin")]
//! let pins = expander.pins_spin_mutex();
//! ```
use crate::expander::{Bank, Mode, PCA9539, PinID};
use crate::guard::RefGuard;
use core::marker::PhantomData;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::i2c::I2c;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;

pub use crate::pin_refreshable::{RefreshableInputPin, RefreshableOutputPin};

/// Container for fetching individual pins
pub struct Pins<'a, I2CT: I2c, RESET: OutputPin, RAWMUTEX: RawMutex> {
    expander: &'a Mutex<RAWMUTEX, Option<PCA9539<I2CT, RESET>>>
}

impl<'a, I2CT: I2c, RESET: OutputPin, RAWMUTEX: RawMutex> Pins<'a, I2CT, RESET, RAWMUTEX> {
    pub fn new(expander: &'a Mutex<RAWMUTEX, Option<PCA9539<I2CT, RESET>>>) -> Self {
        Self {
            expander
        }
    }

    /// Returns an individual pin, which state gets updated synchronously
    /// **The library does not prevent multiple parallel instances of the same pin.**
    pub fn get_pin(&self, bank: Bank, id: PinID) -> Pin<'a, I2CT, RESET, RAWMUTEX, Input, RegularAccessMode> {
        Pin::regular(self.expander, bank, id)
    }

    /// Returns an individual pin, which is using a cached state
    /// The status is explicitly updated. This allows a more efficient status query and assignment,
    /// as the status is only updated once for all pins.
    /// **The library does not prevent multiple parallel instances of the same pin.**
    pub fn get_refreshable_pin(&self, bank: Bank, id: PinID) -> Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode> {
        Pin::refreshable(self.expander, bank, id)
    }
}

/// Marker trait defining how the state of pins is handled.
///
/// Currently there are two modes supported:
/// * Regular: State of the pin is synchronously fetched from I2C bus when calling functions like `is_high()`
/// * Refreshable: State of all pins is refreshed explicitly and functions like `is_high()` are working on a cached state.
/// This reducing the I2C overhead
pub trait AccessMode {}

/// State of the pin is synchronously fetched from I2C bus
pub struct RegularAccessMode {}
impl AccessMode for RegularAccessMode {}

/// Working on cached register state. State of all pins is refreshed explicitly.
pub struct RefreshMode {}
impl AccessMode for RefreshMode {}

/// Indicates the current pin mode. Either Input or Output.
pub trait PinMode {}

/// Input mode
pub struct Input {}
impl PinMode for Input {}

/// Output mode
pub struct Output {}
impl PinMode for Output {}

/// Individual GPIO pin
pub struct Pin<'a, I2CT, RESET, RAWMUTEX, MODE, ACCESS>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    MODE: PinMode,
    ACCESS: AccessMode,
{
    pub(crate) expander: &'a Mutex<RAWMUTEX, Option<PCA9539<I2CT, RESET>>>,
    pub(crate) bank: Bank,
    pub(crate) id: PinID,
    pub(crate) mode: PhantomData<MODE>,
    pub(crate) access_mode: PhantomData<ACCESS>,
    pub(crate) reset: PhantomData<RESET>,
}

impl<'a, I2CT, RESET, RAWMUTEX, ACCESS> Pin<'a, I2CT, RESET, RAWMUTEX, Input, ACCESS>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    ACCESS: AccessMode,
{
    /// Reverses/Resets the input polarity
    pub async fn invert_polarity(&self, invert: bool) -> Result<(), I2CT::Error> {
        self.expander.lock().await.as_mut().unwrap().reverse_polarity(self.bank, self.id, invert).await
    }
}

impl<'a, I2CT, RESET, RAWMUTEX, ACCESS> Pin<'a, I2CT, RESET, RAWMUTEX, Output, ACCESS>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    ACCESS: AccessMode,
{
    /// Returns the current output state, this logic is independent from access mode, as it acts in both
    /// cases on cached register state
    pub(crate) async fn is_pin_output_high(&self) -> bool {
        self.expander.lock().await.as_mut().unwrap().is_pin_output_high(self.bank, self.id)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX, MODE, ACCESS> Pin<'a, I2CT, RESET, RAWMUTEX, MODE, ACCESS>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    ACCESS: AccessMode,
    MODE: PinMode,
{
    /// Switches the pin to the given mode
    pub(crate) async fn change_mode(&self, mode: Mode) -> Result<(), I2CT::Error> {
        self.expander.lock().await.as_mut().unwrap().set_mode(self.bank, self.id, mode).await
    }
}
