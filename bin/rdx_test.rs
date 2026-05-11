use core::time::Duration;

use lcd_menu::{LcdMenu, MenuPage, MenuRow};
use rdx_hal::{RdxDisplay, RotDir};
use syact::units::Factor;

/* LCD-Menu */
    /// Custom app state
    struct MenuState {

    }

    static MAIN_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 3] = [
        MenuRow::new_static(b"RDX -- Hardware test"),
        MenuRow::new_static(b"--------------------"),
        MenuRow::new_static(b"Version")
            .make_link(1)
    ];

    static VERSION_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 8] = [
        MenuRow::new_static(b"RDX -- Hardware test"),
        MenuRow::new_static(b"--------------------"),
        MenuRow::new_static(b"Version: 0.1.0"),
        MenuRow::new_static(b"(c) Sy (Samuel N.)"),
        MenuRow::new_static(b"CircleLab Inc."),
        MenuRow::new_static(b""),
        MenuRow::new_static(b"| Main hardware test"),
        MenuRow::new_static(b"| for the RDX"),
    ];

    static PAGE_LIST : [MenuPage<RdxDisplay, MenuState>; 2] = [
        MenuPage::new(&MAIN_PAGE_ROWS, None),
        MenuPage::new(&VERSION_PAGE_ROWS, Some(0))
    ];
/**/

fn main() {
    // Initialize HAL
    let mut rdx = rdx_hal::Rdx::init()
        .expect("RDX initialization failed!");

    // Create menu
    let state = MenuState {

    };

    let mut menu = LcdMenu::new_with_state(&PAGE_LIST, rdx.display, (20, 4), state);
    menu.start()
        .expect("LCD-Menu initialization failed!");

    // Activate Fan
    rdx.fan.drive_factor(Factor::HALF, syact::units::Direction::CW)
        .expect("Starting motor movement failed!");

    // Print out debug information
    println!("[RDX - Test]");
    println!("|- IO0");
    println!("| |- IO0-0: {}", rdx.io0.0.pin());
    println!("| |- IO0-1: {}", rdx.io0.1.pin());
    println!("| |- IO0-2: {}", rdx.io0.2.pin());
    println!("| |- IO0-3: {}", rdx.io0.3.pin());

    println!("");
    println!("> Program will be kept running, press Ctrl + C to exit");

    let mut sel_once = true;
    
    // Start event-loop
    loop {
        match rdx.rotary_encoder.update()
        {
            RotDir::Clockwise => {
                menu.down()
                    .expect("Menu navigation failed!")
            },
            RotDir::Anticlockwise => {
                menu.up()
                    .expect("Menu navigation failed!")
            },
            RotDir::None => { }
        }

        if rdx.encoder_switch.is_low() {
            if sel_once {
                menu.select()
                    .expect("Menu navigation failed!");
                sel_once = false;
            }
        } else {
            sel_once = true;
        }


        std::thread::sleep(
            Duration::from_micros(100)
        );
    }
}