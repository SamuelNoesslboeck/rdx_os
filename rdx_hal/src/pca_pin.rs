use core::cell::RefCell;
use std::rc::Rc;

use pwm_pca9685::{Channel, Pca9685};
use rppal::i2c::I2c;
use syact::ActuatorError;

#[derive(Debug)]
pub enum PcaPinError {

}

impl embedded_hal::pwm::Error for PcaPinError {
    fn kind(&self) -> embedded_hal::pwm::ErrorKind {
        embedded_hal::pwm::ErrorKind::Other
    }
}

impl Into<ActuatorError> for PcaPinError {
    fn into(self) -> ActuatorError {
        ActuatorError::IOError
    }
}

#[derive(Debug, Clone)]
pub struct PcaPin {
    channel : Channel,
    __pca_ref : Rc<RefCell<Pca9685<I2c>>>
}

impl PcaPin {
    pub fn new(__pca_ref : Rc<RefCell<Pca9685<I2c>>>, channel : Channel) -> Self {
        Self {
            channel, 
            __pca_ref
        }
    }
}

impl embedded_hal::pwm::ErrorType for PcaPin {
    type Error = PcaPinError;
}

impl embedded_hal::pwm::SetDutyCycle for PcaPin {
    fn max_duty_cycle(&self) -> u16 {
        4095
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let mut pca_ref = self.__pca_ref.borrow_mut();
        pca_ref.set_channel_on(self.channel, 0).unwrap();
        pca_ref.set_channel_off(self.channel, duty).unwrap();
        Ok(())
    }
}