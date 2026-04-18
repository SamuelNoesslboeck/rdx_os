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

#[repr(C)]
pub struct RdxStepperCtrls(pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl);

#[repr(C)]
pub struct RdxDcDrivers(pub PcaDcDriver, pub PcaDcDriver, pub PcaDcDriver);

#[repr(C)]
#[derive(Debug)]
pub struct RdxIo(pub Pin, pub Pin, pub Pin, pub Pin);

pub struct Rdx {
    // Motors
    pub step_driver : RdxStepperCtrls,
    pub dc_driver : RdxDcDrivers,
    pub fan : PcaDcDriver,

    pub pca_ref : Rc<RefCell<Pca9685<rppal::i2c::I2c>>>,

    // Plugs
    pub io0 : RdxIo,
    pub io1 : RdxIo
}

impl Rdx {
    pub fn init() -> Self {
        // Create interfaces
        let gpio = Gpio::new().unwrap();            // TODO: Add error type
        let i2c = rppal::i2c::I2c::new().unwrap();      // TODO: Add error type
        
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
        let mut io0 : [MaybeUninit::<Pin>; 4] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 4 {
            io0[i].write(gpio.get(RDX_PIN_IO0[i]).unwrap());    // TODO: Add error
        }

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
            io0: unsafe { core::mem::transmute(io0) },
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