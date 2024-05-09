use crate::expander::{Bank, Mode, PinID, RefreshInputError};
use crate::guard::RefGuard;
use crate::pins::{Input, Output, Pin, RegularAccessMode};
use core::marker::PhantomData;
use embedded_hal_async::i2c::I2c;
use embedded_hal::digital::{InputPin, OutputPin, PinState, StatefulOutputPin};
use embedded_hal::digital::ErrorType;

impl<'a, B, RESET, R> Pin<'a, B, RESET, R, Input, RegularAccessMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    pub fn regular(expander: &'a R, bank: Bank, id: PinID) -> Self {
        Pin {
            expander,
            bus: PhantomData,
            mode: PhantomData,
            access_mode: PhantomData,
            reset: PhantomData,
            bank,
            id,
        }
    }

    pub fn into_input_pin(self) -> Result<Pin<'a, B, RESET, R, Input, RegularAccessMode>, B::Error> {
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

    pub fn into_output_pin(self, state: PinState) -> Result<Pin<'a, B, RESET, R, Output, RegularAccessMode>, B::Error> {
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
        Ok(pin)
    }
}

impl<'a, B, RESET, R> ErrorType for Pin<'a, B, RESET, R, Input, RegularAccessMode>
    where
        B: I2c,
        RESET: OutputPin,
        R: RefGuard<B, RESET>,
{
    type Error = RefreshInputError<B>;
}

impl<'a, B, RESET, R> InputPin for Pin<'a, B, RESET, R, Input, RegularAccessMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{

    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut result = Ok(false);

        self.expander.access(|expander| {
            result = match embassy_futures::block_on(expander.refresh_input_state(self.bank)) {
                Ok(_) => Ok(expander.is_pin_input_high(self.bank, self.id)),
                Err(error) => Err(error),
            };
        });

        result
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<'a, B, RESET, R> ErrorType for Pin<'a, B, RESET, R, Output, RegularAccessMode>
    where
        B: I2c,
        RESET: OutputPin,
        R: RefGuard<B, RESET>,
{
    type Error = RefreshInputError<B>;
}

impl<'a, B, RESET, R> OutputPin for Pin<'a, B, RESET, R, Output, RegularAccessMode>
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
        let mut result = Ok(());


        self.expander.access(|expander| {
            expander.set_state(self.bank, self.id, state == PinState::High);
            result = embassy_futures::block_on(expander.write_output_state(self.bank));
        });

        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(RefreshInputError::I2cError(error)),
        }
    }
}

impl<'a, B, RESET, R> StatefulOutputPin for Pin<'a, B, RESET, R, Output, RegularAccessMode>
where
    B: I2c,
    RESET: OutputPin,
    R: RefGuard<B, RESET>,
{
    /// As this is just acting on cached register data, its in fact Infallible
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_pin_output_high())
    }

    /// As this is just acting on cached register data, its in fact Infallible
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_pin_output_high())
    }
}