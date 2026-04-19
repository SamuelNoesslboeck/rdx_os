use core::time::Duration;

use syact::AsyncActuator;
use syact::units::Factor;

fn main() {
    let mut rdx = rdx_hal::Rdx::init().unwrap();

    rdx.fan.drive_factor(Factor::HALF, syact::units::Direction::CW).unwrap();

    println!("[RDX - Test]");
    println!("|- IO0");
    println!("| |- IO0-0: {}", rdx.io0.0.pin());
    println!("| |- IO0-1: {}", rdx.io0.1.pin());
    println!("| |- IO0-2: {}", rdx.io0.2.pin());
    println!("| |- IO0-3: {}", rdx.io0.3.pin());

    println!("");
    println!("> Program will be kept running, press Ctrl + C to exit");

    loop {
        std::thread::sleep(
            Duration::from_secs(1)
        );
    }
}