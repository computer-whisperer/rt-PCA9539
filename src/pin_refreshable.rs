use crate::expander::{Bank, Mode, PinID, RefreshInputError};
use crate::guard::RefGuard;
use crate::pins::{Input, Output, Pin, RefreshMode};
use core::convert::Infallible;
use core::marker::PhantomData;
use embedded_hal_async::i2c::I2c;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, PinState, StatefulOutputPin};

/// Trait for refreshable pins in output mode
pub trait RefreshableOutputPin {
    type Error;

    /// Updates the output state of all pins of the same bank
    fn update_bank(&self) -> Result<(), Self::Error>;

    /// Updates the output state of all pins (on all banks)
    fn update_all(&self) -> Result<(), Self::Error>;
}

/// Trait for refreshable pins in input mode
pub trait RefreshableInputPin {
    type Error;

    /// Refreshes the input state of all pins of the same bank
    fn refresh_bank(&self) -> Result<(), Self::Error>;

    /// Refreshes the input state of all pins (on all banks)
    fn refresh_all(&self) -> Result<(), Self::Error>;
}

impl<'a, B, RESET, R> Pin<'a, B,  RESET, R, Input, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    pub fn refreshable(expander: &'a R, bank: Bank, id: PinID) -> Self {
        Self {
            expander,
            bus: PhantomData,
            reset: PhantomData,
            bank,
            id,
            access_mode: PhantomData,
            mode: PhantomData,
        }
    }

    /// Refreshes the input state of the given bank
    fn refresh(&self, bank: Bank) -> Result<(), RefreshInputError<B>> {
        let mut result = Ok(());

        self.expander.access(|expander| {
            let fut = expander.refresh_input_state(bank);
            result = embassy_futures::block_on(fut);
        });

        result
    }

    pub fn into_input_pin(self) -> Result<Pin<'a, B, RESET, R, Input, RefreshMode>, B::Error> {
        self.change_mode(Mode::Input)?;

        Ok(Pin {
            expander: self.expander,
            bank: self.bank,
            id: self.id,
            bus: PhantomData,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData
        })
    }

    pub fn into_output_pin(self, state: PinState) -> Result<Pin<'a, B, RESET, R, Output, RefreshMode>, B::Error> {
        self.change_mode(Mode::Output)?;

        let mut pin = Pin {
            expander: self.expander,
            bank: self.bank,
            id: self.id,
            bus: PhantomData,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData
        };

        let _ = pin.set_state(state);
        pin.update_bank()?;
        Ok(pin)
    }
}

impl<'a, B, RESET, R> RefreshableInputPin for Pin<'a, B, RESET, R, Input, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    type Error = RefreshInputError<B>;

    /// Refreshes the input state of all pins of the same bank
    fn refresh_bank(&self) -> Result<(), Self::Error> {
        self.refresh(self.bank)
    }

    /// Refreshes the input state of all pins (on all banks)
    fn refresh_all(&self) -> Result<(), Self::Error> {
        self.refresh(Bank::Bank0)?;
        self.refresh(Bank::Bank1)
    }
}

impl<'a, B, RESET, R> RefreshableOutputPin for Pin<'a, B, RESET, R, Output, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    type Error = B::Error;

    /// Updates the output state of all pins of the same bank
    fn update_bank(&self) -> Result<(), Self::Error> {
        self.update(self.bank)
    }

    /// Updates the output state of all pins (on all banks)
    fn update_all(&self) -> Result<(), Self::Error> {
        self.update(Bank::Bank0)?;
        self.update(Bank::Bank1)
    }
}

impl<'a, B, RESET, R> Pin<'a, B, RESET, R, Output, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    /// Writes the output state of the given bank
    fn update(&self, bank: Bank) -> Result<(), B::Error> {
        let mut result = Ok(());

        self.expander.access(|expander| {
            let fut = expander.write_output_state(bank);
            result = embassy_futures::block_on(fut);
        });

        result
    }
}

impl<'a, B, RESET, R> ErrorType for Pin<'a, B, RESET, R, Input, RefreshMode>
    where
        B: I2c,
        RESET: OutputPin,
        R: RefGuard<B, RESET> {
    type Error = Infallible;
}

impl<'a, B, RESET, R> InputPin for Pin<'a, B, RESET, R, Input, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{

    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut state = false;

        self.expander.access(|expander| {
            state = expander.is_pin_input_high(self.bank, self.id);
        });

        Ok(state)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<'a, B, RESET, R> ErrorType for Pin<'a, B, RESET, R, Output, RefreshMode>
    where
        B: I2c,
        RESET: OutputPin,
        R: RefGuard<B, RESET>,
{
    type Error = Infallible;
}

impl<'a, B, RESET, R> OutputPin for Pin<'a, B, RESET, R, Output, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::Low)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_state(PinState::High)
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        self.expander.access(|expander| {
            expander.set_state(self.bank, self.id, state == PinState::High);
        });

        Ok(())
    }
}

impl<'a, B, RESET, R> StatefulOutputPin for Pin<'a, B, RESET, R, Output, RefreshMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_pin_output_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_pin_output_high())
    }
}