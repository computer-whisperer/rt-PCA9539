use crate::expander::{PCA9539, Bank, Mode, PinID, RefreshInputError};
use crate::pins::{Input, Output, Pin, RefreshMode};
use core::convert::Infallible;
use core::marker::PhantomData;
use embedded_hal_async::i2c::I2c;
use embedded_hal::digital::{InputPin, OutputPin, PinState, StatefulOutputPin};
use embedded_hal::digital;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use crate::digital_hal_async::{InputPinAsync, OutputPinAsync};
use crate::digital_hal_async;


/// Trait for refreshable pins in output mode
pub trait RefreshableOutputPin {
    type Error;

    /// Updates the output state of all pins of the same bank
    async fn update_bank(&self) -> Result<(), Self::Error>;

    /// Updates the output state of all pins (on all banks)
    async fn update_all(&self) -> Result<(), Self::Error>;
}

/// Trait for refreshable pins in input mode
pub trait RefreshableInputPin {
    type Error;

    /// Refreshes the input state of all pins of the same bank
    async fn refresh_bank(&self) -> Result<(), Self::Error>;

    /// Refreshes the input state of all pins (on all banks)
    async fn refresh_all(&self) -> Result<(), Self::Error>;
}

impl<'a, I2CT, RESET, RAWMUTEX> Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    pub fn refreshable(expander: &'a Mutex<RAWMUTEX, Option<PCA9539<I2CT, RESET>>>, bank: Bank, id: PinID) -> Self {
        Self {
            expander,
            reset: PhantomData,
            bank,
            id,
            access_mode: PhantomData,
            mode: PhantomData,
        }
    }

    /// Refreshes the input state of the given bank
    async fn refresh(&self, bank: Bank) -> Result<(), RefreshInputError<I2CT>> {
        let mut expander = self.expander.lock().await;
        expander.as_mut().unwrap().refresh_input_state(bank).await
    }

    pub async fn into_input_pin(self) -> Result<Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>, I2CT::Error> {
        self.change_mode(Mode::Input).await?;

        Ok(Pin {
            expander: self.expander,
            bank: self.bank,
            id: self.id,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData
        })
    }

    pub async fn into_output_pin(self, state: PinState) -> Result<Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>, I2CT::Error> {
        self.change_mode(Mode::Output).await?;

        let mut pin = Pin {
            expander: self.expander,
            bank: self.bank,
            id: self.id,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData
        };

        let _ = pin.set_state_async(state).await;
        pin.update_bank().await?;
        Ok(pin)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> RefreshableInputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    type Error = RefreshInputError<I2CT>;

    /// Refreshes the input state of all pins of the same bank
    async fn refresh_bank(&self) -> Result<(), Self::Error> {
        self.refresh(self.bank).await
    }

    /// Refreshes the input state of all pins (on all banks)
    async fn refresh_all(&self) -> Result<(), Self::Error> {
        self.refresh(Bank::Bank0).await?;
        self.refresh(Bank::Bank1).await
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> RefreshableOutputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    type Error = I2CT::Error;

    /// Updates the output state of all pins of the same bank
    async fn update_bank(&self) -> Result<(), Self::Error> {
        self.update(self.bank).await
    }

    /// Updates the output state of all pins (on all banks)
    async fn update_all(&self) -> Result<(), Self::Error> {
        self.update(Bank::Bank0).await?;
        self.update(Bank::Bank1).await
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    /// Writes the output state of the given bank
    async fn update(&self, bank: Bank) -> Result<(), I2CT::Error> {
        let mut expander = self.expander.lock().await;
        expander.as_mut().unwrap().write_output_state(bank).await
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> digital_hal_async::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    type Error = Infallible;
}

impl<'a, I2CT, RESET, RAWMUTEX> digital::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex,
{
    type Error = Infallible;
}


impl<'a, I2CT, RESET, RAWMUTEX> InputPinAsync for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{

    async fn is_high_async(&mut self) -> Result<bool, Self::Error> {
        let mut expander = self.expander.lock().await;
        Ok(expander.as_mut().unwrap().is_pin_input_high(self.bank, self.id))
    }

    async fn is_low_async(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high_async().await?)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> InputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RefreshMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex,
{

    fn is_high(&mut self) -> Result<bool, Self::Error> {
        embassy_futures::block_on(self.is_high_async())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> digital_hal_async::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    type Error = Infallible;
}

impl<'a, I2CT, RESET, RAWMUTEX> digital::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex,
{
    type Error = Infallible;
}

impl<'a, I2CT, RESET, RAWMUTEX> OutputPinAsync for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
{
    async fn set_low_async(&mut self) -> Result<(), Self::Error> {
        self.set_state_async(PinState::Low).await
    }

    async fn set_high_async(&mut self) -> Result<(), Self::Error> {
        self.set_state_async(PinState::High).await
    }

    async fn set_state_async(&mut self, state: PinState) -> Result<(), Self::Error> {
        let mut expander = self.expander.lock().await;
        Ok(expander.as_mut().unwrap().set_state(self.bank, self.id, state == PinState::High))
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> OutputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::Low)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::High)
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        embassy_futures::block_on(self.set_state_async(state))
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> StatefulOutputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RefreshMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(embassy_futures::block_on(self.is_pin_output_high()))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }
}