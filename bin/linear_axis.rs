use syact::meas::EndStop;

fn main() {
    let rdx = rdx_hal::Rdx::init();

    let endstop = EndStop::new(false, Some(syact::units::Direction::CCW), rdx.io0.0.into_input());

    println!("Successfully initialized!");
}