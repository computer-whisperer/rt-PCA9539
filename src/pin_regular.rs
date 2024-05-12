use crate::expander::{Bank, Mode, PCA9539, PinID, RefreshInputError};
use crate::pins::{Input, Output, PinMode, Pin, RegularAccessMode};
use core::marker::PhantomData;
use embedded_hal_async::i2c::I2c;
use embedded_hal::digital::{InputPin, OutputPin, PinState, StatefulOutputPin};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use crate::digital_hal_async::{InputPinAsync, OutputPinAsync};
use crate::digital_hal_async;

impl<'a, I2CT, RESET, RAWMUTEX> Pin<'a, I2CT, RESET, RAWMUTEX, Input, RegularAccessMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex
{
    pub fn regular(expander: &'a Mutex<RAWMUTEX, Option<PCA9539<I2CT, RESET>>>, bank: Bank, id: PinID) -> Self {
        Pin {
            expander,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData,
            bank,
            id,
        }
    }

    pub async fn into_input_pin(self) -> Result<Pin<'a, I2CT, RESET, RAWMUTEX, Input, RegularAccessMode>, I2CT::Error> {
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

    pub async fn into_output_pin(self, state: PinState) -> Result<Pin<'a, I2CT, RESET, RAWMUTEX, Output, RegularAccessMode>, RefreshInputError<I2CT>> {
        self.change_mode(Mode::Output).await.map_err(|e| RefreshInputError::<I2CT>::WriteError(e));

        let mut pin = Pin {
            expander: self.expander,
            bank: self.bank,
            id: self.id,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData
        };

        pin.set_state_async(state).await?;
        Ok(pin)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX, MODE> digital_hal_async::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, MODE, RegularAccessMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    MODE: PinMode
{
    type Error = RefreshInputError<I2CT>;
}

impl<'a, I2CT, RESET, RAWMUTEX, MODE> embedded_hal::digital::ErrorType for Pin<'a, I2CT, RESET, RAWMUTEX, MODE, RegularAccessMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex,
    MODE: PinMode
{
    type Error = RefreshInputError<I2CT>;
}


impl<'a, I2CT, RESET, RAWMUTEX> InputPinAsync for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RegularAccessMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex
{

    async fn is_high_async(&mut self) -> Result<bool, Self::Error> {
        let mut expander = self.expander.lock().await;
        expander.as_mut().unwrap().refresh_input_state(self.bank).await?;
        Ok(expander.as_mut().unwrap().is_pin_input_high(self.bank, self.id))
    }

    async fn is_low_async(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high_async().await?)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> OutputPinAsync for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RegularAccessMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex
{

    async fn set_low_async(&mut self) -> Result<(), Self::Error> {
        self.set_state_async(PinState::Low).await
    }

    async fn set_high_async(&mut self) -> Result<(), Self::Error> {
        self.set_state_async(PinState::High).await
    }

    async fn set_state_async(&mut self, state: PinState) -> Result<(), Self::Error> {
        let mut expander = self.expander.lock().await;
        expander.as_mut().unwrap().set_state(self.bank, self.id, state == PinState::High);
        expander.as_mut().unwrap().write_output_state(self.bank).await.map_err(|e| RefreshInputError::I2cError(e))
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> InputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Input, RegularAccessMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        embassy_futures::block_on(self.is_high_async())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> OutputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RegularAccessMode>
    where
        I2CT: I2c,
        RESET: OutputPin,
        RAWMUTEX: RawMutex
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        embassy_futures::block_on(self.set_low_async())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        embassy_futures::block_on(self.set_high_async())
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        embassy_futures::block_on(self.set_state_async(state))
    }
}

impl<'a, I2CT, RESET, RAWMUTEX> StatefulOutputPin for Pin<'a, I2CT, RESET, RAWMUTEX, Output, RegularAccessMode>
where
    I2CT: I2c,
    RESET: OutputPin,
    RAWMUTEX: RawMutex
{
    /// As this is just acting on cached register data, its in fact Infallible
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(embassy_futures::block_on(self.is_pin_output_high()))
    }

    /// As this is just acting on cached register data, its in fact Infallible
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }
}