use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::str::from_utf8;
use std::rc::Rc;

use embedded_hal_bus::i2c::RefCellDevice;
use lcd_lcm1602_i2c::sync_lcd::Lcd;
use lcd_menu::LcdDisplay;
use rotary_encoder_embedded::RotaryEncoder;
use rotary_encoder_embedded::standard::StandardMode;
use rppal::gpio::{Gpio, InputPin, OutputPin, Pin};
use pwm_pca9685::Pca9685;
use rppal::hal::Delay;
use rppal::i2c::I2c;
use syact::PwmDcDriver;
use systep::GenericPulseCtrl;

use crate::defines::*;

// Submodules
pub mod defines;

mod pca_pin;
pub use pca_pin::*;

pub type RotDir = rotary_encoder_embedded::Direction;

pub type SharedI2c<'a> = RefCellDevice<'a, I2c>;

/// Stepper motor control type with two PWM pins from the RPi (have the same type, no generics)
pub type RPiStepperCtrl = GenericPulseCtrl<OutputPin, OutputPin, Delay>;
/// Servo motor connected to the Pca9685 PWM board
pub type PcaServo = i32;    // TODO: Add servo type
/// Dc motor driver connected to the Pca9685 PWM board
pub type PcaDcDriver<'a> = PwmDcDriver<PcaPin<SharedI2c<'a>>, PcaPin<SharedI2c<'a>>>;

#[repr(C)]      // Required for init (Rust could change tuple memory layout)
pub struct RdxStepperCtrls(pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl, pub RPiStepperCtrl);

#[repr(C)]
pub struct RdxDcDrivers<'a>(pub PcaDcDriver<'a>, pub PcaDcDriver<'a>, pub PcaDcDriver<'a>);

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

pub struct RdxDisplay<'a>(pub Lcd<'a, 4, 20, RefCellDevice<'a, I2c>, Delay>);

impl<'a> LcdDisplay for RdxDisplay<'a> {
    type Error = rppal::i2c::Error;

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.0.clear()
    }

    fn move_cursor(&mut self, col_id : usize, row_id : usize) {
        self.0.set_cursor(row_id as u8, col_id as u8)
            .expect("Failed to move cursor")        // TODO: Update as soon as library allows to return an error here
    }

    fn write(&mut self, text : &[u8]) -> Result<(), Self::Error> {
        self.0.write_str(
            from_utf8(text)
                .expect("UTF8 Conversion failed!")
        )
    }
}

/// ### RDX-HAL
/// 
/// Provides easy access to all components of the RDX by initializing them with the right hardware parameters. The components 
/// are created in tuple structures, so ownership can be passed as required
pub struct Rdx<'a> {
    /* Motors */
        pub step_driver : RdxStepperCtrls,
        pub dc_driver : RdxDcDrivers<'a>,
        pub fan : PcaDcDriver<'a>,
    /**/

    /* Plugs */
        pub io0 : RdxIo,
        pub io1 : RdxIo,

        // TODO: Include misc?
    /**/

    /* User panel */
        pub display : RdxDisplay<'a>,

        pub rotary_encoder : RotaryEncoder<StandardMode, InputPin, InputPin>, 
        pub encoder_switch : InputPin, 
    /**/

    /* Periphals */
        pub i2c: &'static RefCell<I2c>,

        pub gpio : Gpio,
        pub pca9685 : Rc<RefCell<Pca9685<RefCellDevice<'a, I2c>>>>,
    /**/
}

impl<'a> Rdx<'a> {
    /// Initializes the RDX with the correct hardware parameters
    /// 
    /// ### Error
    /// 
    /// Returns `RdxError` if any of the components fails to initialize correctly
    pub fn init() -> Result<Self, RdxError> {
        // Create interfaces
        let gpio = Gpio::new().map_err(|err| RdxError::GpioError(err))?;        
        let i2c : &'static RefCell<I2c> = Box::leak(Box::new(
            RefCell::new(I2c::new().map_err(|err| RdxError::I2cError(err))?)
        ));

        // Components
        let mut pca = Pca9685::new(
            RefCellDevice::new(&i2c), 
            RDX_I2C_ADDR_PCA9685
        ).map_err(|err| RdxError::PcaError(err))?;
        pca.enable().unwrap();

        let pca9685 = Rc::new(RefCell::new(pca)); 

        let mut step_driver : [MaybeUninit::<RPiStepperCtrl>; 4] = unsafe { 
            // Initializing this way is safe, as it is not read until it's created
           MaybeUninit::uninit().assume_init()
        };
        
        for i in 0 .. 4 {
            step_driver[i].write(RPiStepperCtrl::new(RDX_DATA_SC, 
                gpio.get(RDX_PIN_DIR[i])
                    .map_err(|err| RdxError::GpioError(err))?.into_output(), 
                gpio.get(RDX_PIN_STEP[i])
                    .map_err(|err| RdxError::GpioError(err))?.into_output(),
                Delay::new()
            ));
        }

        let mut dc_driver : [MaybeUninit::<PcaDcDriver>; 3] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        for i in 0 .. 3 {
            dc_driver[i].write(PcaDcDriver::init(
                PcaPin::new(pca9685.clone(), RDX_CHANNEL_DC[i*2]), 
                PcaPin::new(pca9685.clone(), RDX_CHANNEL_DC[i*2 + 1])
            ));
        }

        /* Plugs */
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
        /**/

        /* User panel */
            let delay : &'static mut Delay = Box::leak(Box::new(Delay::new()));
            let i2c_proxy = Box::leak(Box::new(RefCellDevice::new(&i2c)));
            let display = RdxDisplay(
                Lcd::new(i2c_proxy, delay)
                    .with_address(RDX_I2C_ADDR_LCD)
                    .with_cursor_on(false)
                    .with_cursor_blink(false)
                    .init()
                    .map_err(|err| RdxError::I2cError(err))?
            );

            let rotary_encoder = RotaryEncoder::new(
                gpio.get(RDX_PIN_ROT_DT)
                    .map_err(|err| RdxError::GpioError(err))?.into_input_pullup(),
                gpio.get(RDX_PIN_ROT_CL)
                    .map_err(|err| RdxError::GpioError(err))?.into_input_pullup()
            ).into_standard_mode();

            let encoder_switch = gpio.get(RDX_PIN_ROT_SW)
                .map_err(|err| RdxError::GpioError(err))?.into_input_pullup();
        /**/

        // Creating HAL
        Ok(Self {
            step_driver: unsafe { core::mem::transmute(step_driver) },  
            dc_driver: unsafe { core::mem::transmute(dc_driver) },

            fan: PcaDcDriver::init(
                PcaPin::new(pca9685.clone(), RDX_CHANNEL_FAN[0]), 
                PcaPin::new(pca9685.clone(), RDX_CHANNEL_FAN[1])
            ),

            /* Plugs */
                io0: unsafe { core::mem::transmute(io0) },
                io1: unsafe { core::mem::transmute(io1) },
            /**/

            /* User panel */
                display,

                rotary_encoder,
                encoder_switch,
            /**/    

            /* Periphals */
                i2c,

                gpio,
                pca9685
            /**/
        })
    }
}