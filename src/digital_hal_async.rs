use core::convert::Infallible;
use embedded_hal::digital::{OutputPin, InputPin, PinState};

pub trait Error : core::fmt::Debug  {}

impl Error for Infallible {}

pub trait ErrorType {
    type Error : Error;
}

pub trait OutputPinAsync : ErrorType {
    async fn set_low_async(&mut self) -> Result<(), Self::Error>;

    async fn set_high_async(&mut self) -> Result<(), Self::Error>;

    async fn set_state_async(&mut self, state: PinState) -> Result<(), Self::Error>;
}

pub trait InputPinAsync : ErrorType {
    async fn is_high_async(&mut self) -> Result<bool, Self::Error>;

    async fn is_low_async(&mut self) -> Result<bool, Self::Error>;
}