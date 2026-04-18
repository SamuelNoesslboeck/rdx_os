use core::cell::RefCell;
use core::mem::MaybeUninit;
use std::rc::Rc;

use rppal::gpio::{Gpio, Pin, OutputPin};
use pwm_pca9685::Pca9685;
use syact::PWMDcDriver;
use systep::GenericPulseCtrl;

use crate::defines::*;

// Submodules
pub mod defines;

mod pca_pin;
pub use pca_pin::*;

/// Stepper motor control type with two PWM pins from the RPi (have the same type, no generics)
pub type RPiStepperCtrl = GenericPulseCtrl<OutputPin, OutputPin>;
pub type PcaServo = i32;
pub type PcaDcDriver = PWMDcDriver<PcaPin, PcaPin>;

pub struct RDX {
    // Motors
    pub step_driver : [RPiStepperCtrl; 4],
    pub dc_driver : [PcaDcDriver; 3],
    pub fan : PcaDcDriver,

    pub pca_ref : Rc<RefCell<Pca9685<rppal::i2c::I2c>>>,

    // Plugs
    pub io1 : [Pin; 4],
    // pub io2 : [Pin; 4]
}

impl RDX {
    pub fn init() -> Self {
        // Create interfaces
        let gpio = Gpio::new().unwrap();    // TODO: Add error type
        let i2c = rppal::i2c::I2c::new().unwrap(); // TODO: Add error type
        
        // Components
        let mut pca = Pca9685::new(i2c, RDX_PCA9685_ADDR).unwrap();    // TODO: Add error type
        pca.enable().unwrap();

        let pca_ref = Rc::new(RefCell::new(pca)); 

        let mut step_driver : [MaybeUninit::<RPiStepperCtrl>; 4] = unsafe { 
            // Initializing this way is safe, as it is not read until it's created
           MaybeUninit::uninit().assume_init()
        };
        
        for i in 0 .. 4 {
            step_driver[i].write(RPiStepperCtrl::new(RDX_SC_DATA, 
                gpio.get(RDX_PIN_DIR[i]).unwrap().into_output(), 
                gpio.get(RDX_PIN_STEP[i]).unwrap().into_output()
            ));
        }

        let mut dc_driver : [MaybeUninit::<PcaDcDriver>; 3] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 3 {
            dc_driver[i].write(PcaDcDriver::init(
                PcaPin::new(pca_ref.clone(), RDX_DC_CHANNEL[i*2]), 
                PcaPin::new(pca_ref.clone(), RDX_DC_CHANNEL[i*2 + 1]), 
                RDX_DC_MAX_SPEED
            ));
        }

        // IO Plugs
        let mut io1 : [MaybeUninit::<Pin>; 4] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 4 {
            io1[i].write(gpio.get(RDX_PIN_IO1[i]).unwrap());    // TODO: Add error
        }

        // Creating HAL
        Self {
            step_driver: unsafe { core::mem::transmute(step_driver) },  
            dc_driver: unsafe { core::mem::transmute(dc_driver) },
            io1: unsafe { core::mem::transmute(io1) },
            fan: PcaDcDriver::init(
                PcaPin::new(pca_ref.clone(), RDX_FAN_CHANNEL[0]), 
                PcaPin::new(pca_ref.clone(), RDX_FAN_CHANNEL[1]), 
                RDX_DC_MAX_SPEED
            ),
            pca_ref
        }
    }
}