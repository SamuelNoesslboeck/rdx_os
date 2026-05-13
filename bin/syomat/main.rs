use syact::Interruptible;
use syact::comps::LinearAxis;
use syact::meas::EndStop;
use syact::units::metric::Millimeters;
use systep::builder::StartStopBuilder;
use systep::{StepperConfig, StepperData, StepperMotor};

// Consts
pub const STEPPER_CONFIG : StepperConfig = StepperConfig::VOLT24_NO_OVERLOAD;


fn main() {
    let rdx = rdx_hal::Rdx::init()  
        .expect("Setting up RDX failed!");

    // let mut x = LinearAxis::new_belt_axis(
    //     StepperMotor::<StartStopBuilder, _>::new_advanced(rdx.stepper_driver.0, StepperData::MOT_17HE15_1504S, STEPPER_CONFIG)
    //         .expect("Setting up stepper motor failed!"), 
    //     Millimeters(4.0)
    // ).add_interruptor_inline(
    //     Box::new(EndStop::new(false, Some(syact::units::Direction::CCW)))
    // );
}