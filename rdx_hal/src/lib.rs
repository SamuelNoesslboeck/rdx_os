use core::cell::RefCell;
use core::mem::MaybeUninit;
use std::rc::Rc;

use rppal::gpio::{Gpio, Pin, OutputPin};
use pwm_pca9685::Pca9685;
use syact::PwmDcDriver;
use systep::GenericPulseCtrl;

use crate::defines::*;

// Submodules
pub mod defines;

mod pca_pin;
pub use pca_pin::*;

/// Stepper motor control type with two PWM pins from the RPi (have the same type, no generics)
pub type RPiStepperCtrl = GenericPulseCtrl<OutputPin, OutputPin>;
/// Servo motor connected to the Pca9685 PWM board
pub type PcaServo = i32;    // TODO: Add servo type
/// Dc motor driver connected to the Pca9685 PWM board
pub type PcaDcDriver = PwmDcDriver<PcaPin, PcaPin>;

#[repr(C)]      // Required for init (Rust could change tuple memory layout)
pub struct RdxStepperCtrls(pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl);

#[repr(C)]
pub struct RdxDcDrivers(pub PcaDcDriver, pub PcaDcDriver, pub PcaDcDriver);

#[repr(C)]
pub struct RdxServos(pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo, pub PcaServo);

#[repr(C)]
#[derive(Debug)]
pub struct RdxIo(pub Pin, pub Pin, pub Pin, pub Pin);

#[derive(Debug)]
pub enum RdxError {
    // Interface errors
    GpioError(rppal::gpio::Error),
    I2cError(rppal::i2c::Error),

    // Device errors
    PcaError(pwm_pca9685::Error<rppal::i2c::Error>)
}

/// ### RDX-HAL
/// 
/// Provides easy access to all components of the RDX by initializing them with the right hardware parameters. The components 
/// are created in tuple structures, so ownership can be passed as required
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
    /// Initializes the RDX with the correct hardware parameters
    /// 
    /// ### Error
    /// 
    /// Returns `RdxError` if any of the components fails to initialize correctly
    pub fn init() -> Result<Self, RdxError> {
        // Create interfaces
        let gpio = Gpio::new().map_err(|err| RdxError::GpioError(err))?;        
        let i2c = rppal::i2c::I2c::new().map_err(|err| RdxError::I2cError(err))?;
        
        // Components
        let mut pca = Pca9685::new(i2c, RDX_PCA9685_ADDR)
            .map_err(|err| RdxError::PcaError(err))?;
        pca.enable().unwrap();

        let pca_ref = Rc::new(RefCell::new(pca)); 

        let mut step_driver : [MaybeUninit::<RPiStepperCtrl>; 4] = unsafe { 
            // Initializing this way is safe, as it is not read until it's created
           MaybeUninit::uninit().assume_init()
        };
        
        for i in 0 .. 4 {
            step_driver[i].write(RPiStepperCtrl::new(RDX_SC_DATA, 
                gpio.get(RDX_PIN_DIR[i])
                    .map_err(|err| RdxError::GpioError(err))?.into_output(), 
                gpio.get(RDX_PIN_STEP[i])
                    .map_err(|err| RdxError::GpioError(err))?.into_output()
            ));
        }

        let mut dc_driver : [MaybeUninit::<PcaDcDriver>; 3] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 3 {
            dc_driver[i].write(PcaDcDriver::init(
                PcaPin::new(pca_ref.clone(), RDX_DC_CHANNEL[i*2]), 
                PcaPin::new(pca_ref.clone(), RDX_DC_CHANNEL[i*2 + 1])
            ));
        }

        // IO Plugs
        let mut io0 : [MaybeUninit::<Pin>; 4] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 4 {
            io0[i].write(gpio.get(RDX_PIN_IO0[i]).map_err(|err| RdxError::GpioError(err))?);
        }

        let mut io1 : [MaybeUninit::<Pin>; 4] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 4 {
            io1[i].write(gpio.get(RDX_PIN_IO1[i]).map_err(|err| RdxError::GpioError(err))?);
        }

        // Creating HAL
        Ok(Self {
            step_driver: unsafe { core::mem::transmute(step_driver) },  
            dc_driver: unsafe { core::mem::transmute(dc_driver) },
            io0: unsafe { core::mem::transmute(io0) },
            io1: unsafe { core::mem::transmute(io1) },
            fan: PcaDcDriver::init(
                PcaPin::new(pca_ref.clone(), RDX_FAN_CHANNEL[0]), 
                PcaPin::new(pca_ref.clone(), RDX_FAN_CHANNEL[1])
            ),
            pca_ref
        })
    }
}