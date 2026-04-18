fn main() {
    let rdx = rdx_hal::Rdx::init().unwrap();

    println!("[RDX - Test]");
    println!("|- IO0");
    println!("| |- IO0-0: {}", rdx.io0.0.pin());
    println!("| |- IO0-1: {}", rdx.io0.1.pin());
    println!("| |- IO0-2: {}", rdx.io0.2.pin());
    println!("| |- IO0-3: {}", rdx.io0.3.pin());
}