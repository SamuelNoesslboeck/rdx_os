use pwm_pca9685::Pca9685;
use syact::PWMDcDriver;
use systep::GenericPulseCtrl;

use crate::defines::RDX_PCA9685_ADDR;

// Submodules
pub mod defines;

/// Stepper motor control type with two PWM pins from the RPi (have the same type, no generics)
pub type RDXStepperCtrl = GenericPulseCtrl<>;

pub struct RDX {
    pub step_driver : [RDXStepperCtrl; 4],
    
    __pca : Pca9685<>
}

impl RDX {
    pub fn init() -> Self {
        let mut pca = Pca9685::new(todo!(), RDX_PCA9685_ADDR)?;
        
        pca.enable();

        Self {
            step_driver: [

            ]
        }
    }
}