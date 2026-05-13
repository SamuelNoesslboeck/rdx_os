use core::time::Duration;

use futures::executor::block_on;
use lcd_menu::{LcdMenu, MenuPage, MenuRow};
use rdx_hal::{PcaDcDriver, RPiStepperCtrl, RdxDisplay, RotDir};
use rppal::gpio::{Level, OutputPin};
use syact::SyncActuator;
use syact::units::{Direction, Factor, Radians};
use systep::{StepperConfig, StepperData, StepperMotor};
use systep::builder::StartStopBuilder;

pub const STEPPER_FAC : Factor = Factor::MAX;

/* LCD-Menu */
    /// Custom app state
    struct MenuState<'a> {
        // Stepper motors
        pub stepper_i : StepperMotor<StartStopBuilder, RPiStepperCtrl>,
        pub stepper_ii : StepperMotor<StartStopBuilder, RPiStepperCtrl>,
        pub stepper_iii : StepperMotor<StartStopBuilder, RPiStepperCtrl>,
        pub stepper_iv : StepperMotor<StartStopBuilder, RPiStepperCtrl>,

        pub stepper_enable_pin : OutputPin,
        pub stepper_enabled : bool,

        // Inputs
        pub io10_state : bool,
        pub io11_state : bool,
        pub io12_state : bool,
        pub io13_state : bool,

        // Fan
        pub fan : PcaDcDriver<'a>,
        pub fan_active : bool
    }

    static MAIN_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 6] = [
        MenuRow::new_static(b"      RDX - OS      "),
        MenuRow::new_static(b"--------------------"),
        MenuRow::new_static(b"Control")
            .make_link(1),
        MenuRow::new_static(b"Inputs")
            .make_link(2),
        MenuRow::new_static(b"Options")
            .make_link(3),
        MenuRow::new_static(b"Infos")
            .make_link(4)
    ];

    static CONTROL_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 6] = [ 
        MenuRow::new_static(b"##  CONTROL PAGE  ##"),
        MenuRow::new_static(b"Stepper-I")
            .make_option(|menu| {
                block_on(
                    menu.state.stepper_i.drive_rel(Radians(0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }, |menu| {
                block_on(
                    menu.state.stepper_i.drive_rel(Radians(-0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }),
        MenuRow::new_static(b"Stepper-II")
            .make_option(|menu| {
                block_on(
                    menu.state.stepper_ii.drive_rel(Radians(0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }, |menu| {
                block_on(
                    menu.state.stepper_ii.drive_rel(Radians(-0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }),
        MenuRow::new_static(b"Stepper-III")
            .make_option(|menu| {
                block_on(
                    menu.state.stepper_iii.drive_rel(Radians(0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }, |menu| {
                block_on(
                    menu.state.stepper_iii.drive_rel(Radians(-0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }),
        MenuRow::new_static(b"Stepper-IV")
            .make_option(|menu| {
                block_on(
                    menu.state.stepper_iv.drive_rel(Radians(0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }, |menu| {
                block_on(
                    menu.state.stepper_iv.drive_rel(Radians(-0.5), STEPPER_FAC)
                ).expect("Moving stepper failed!")
            }),
        MenuRow::new_static(b"Back")
            .make_return()
    ];

    static INPUT_PAGE_ROWS: [MenuRow<RdxDisplay, MenuState>; 3] = [
        MenuRow::new_static(b"###  INPUT PAGE  ###"),
        MenuRow::new_static(b"| IO1 [    ]"),
        MenuRow::new_static(b"Back")
            .make_return()
    ];

    static OPTIONS_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 4] = [
        MenuRow::new_static(b"##  OPTIONS PAGE  ##"),
        MenuRow::new_dynamic(|menu: &mut LcdMenu<'_, RdxDisplay<'static>, MenuState<'static>>| {
            menu.write_str(b"Motors active ")?;
            menu.write_box(menu.state.stepper_enabled)
        }).make_button(|menu| {
            menu.state.stepper_enable_pin.toggle();
            menu.state.stepper_enabled = !menu.state.stepper_enabled
        }),
            
        MenuRow::new_dynamic(|menu: &mut LcdMenu<'_, RdxDisplay<'static>, MenuState<'static>>| {
            menu.write_str(b"Fan active    ")?;
            menu.write_box(menu.state.fan_active)
        }).make_button(|menu| {
            if menu.state.fan_active {
                menu.state.fan.stop()
                    .expect("Fan control failed!")
            } else {
                menu.state.fan.drive_factor(Factor::HALF, Direction::CW)
                    .expect("Fan control failed!")
            }

            menu.state.fan_active = !menu.state.fan_active
        }),
        MenuRow::new_static(b"Back")
            .make_return()
    ];

    static INFO_PAGE_ROWS : [MenuRow<RdxDisplay, MenuState>; 11] = [
        MenuRow::new_static(b"###  INFO  PAGE  ###"),
        MenuRow::new_static(b"Version: 0.1.0"),
        MenuRow::new_static(b"(c) Sy (Samuel N.)"),
        MenuRow::new_static(b"CircleLab Inc."),
        MenuRow::new_static(b""),
        MenuRow::new_static(b"| Main hardware test"),
        MenuRow::new_static(b"| for the RDX"),
        MenuRow::new_static(b""),
        MenuRow::new_static(b"\"Boxsy 2.0 - The"),
        MenuRow::new_static(b"return of the servo\""),
        MenuRow::new_static(b"  - An idiot")
    ];

    static PAGE_LIST : [MenuPage<RdxDisplay, MenuState>; 5] = [
        MenuPage::new(&MAIN_PAGE_ROWS, None),
        MenuPage::new(&CONTROL_PAGE_ROWS, Some(0)),
        MenuPage::new(&INPUT_PAGE_ROWS, Some(0)),
        MenuPage::new(&OPTIONS_PAGE_ROWS, Some(0)),
        MenuPage::new(&INFO_PAGE_ROWS, Some(0))
    ];
/**/

fn main() {
    // Initialize HAL
    let mut rdx = rdx_hal::Rdx::init()
        .expect("RDX initialization failed!");

    // Activate Fan
    rdx.fan.drive_factor(Factor::HALF, Direction::CW)
        .expect("Starting motor movement failed!");

    // Create menu
    let state = MenuState {
        // Stepper motors
        stepper_i: StepperMotor::new_advanced(
            rdx.stepper_driver.0, 
            StepperData::MOT_17HE15_1504S, 
            StepperConfig::VOLT24_NO_OVERLOAD
        ).expect("Initialzing stepper motor failed!"),
        stepper_ii: StepperMotor::new_advanced(
            rdx.stepper_driver.1, 
            StepperData::MOT_17HE15_1504S, 
            StepperConfig::VOLT24_NO_OVERLOAD
        ).expect("Initialzing stepper motor failed!"),
        stepper_iii: StepperMotor::new_advanced(
            rdx.stepper_driver.2, 
            StepperData::MOT_17HE15_1504S, 
            StepperConfig::VOLT24_NO_OVERLOAD
        ).expect("Initialzing stepper motor failed!"),
        stepper_iv: StepperMotor::new_advanced(
            rdx.stepper_driver.3, 
            StepperData::MOT_17HE15_1504S, 
            StepperConfig::VOLT24_NO_OVERLOAD
        ).expect("Initialzing stepper motor failed!"),

        stepper_enable_pin: rdx.stepper_enable,
        stepper_enabled: true,

        io10_state: false,
        io11_state: false,
        io12_state: false,
        io13_state: false,

        // Fan
        fan: rdx.fan,
        fan_active: true
    }; 

    let mut menu = LcdMenu::new_with_state(&PAGE_LIST, rdx.display, (20, 4), state);
    menu.start()
        .expect("LCD-Menu initialization failed!");

    // Print out debug information
    println!("[RDX - Test]");
    println!("|- IO0");
    println!("| |- IO0-0: {}", rdx.io1.0.pin());
    println!("| |- IO0-1: {}", rdx.io1.1.pin());
    println!("| |- IO0-2: {}", rdx.io1.2.pin());
    println!("| |- IO0-3: {}", rdx.io1.3.pin());

    println!("");
    println!("> Program will be kept running, press Ctrl + C to exit");

    let mut sel_once = true;

    let io10 = rdx.io1.0.into_input();
    let io11 = rdx.io1.1.into_input();
    let io12 = rdx.io1.2.into_input();
    let io13 = rdx.io1.3.into_input();
    
    // Start event-loop
    loop {
        // Update encoder movements
        match rdx.rotary_encoder.update()
        {
            RotDir::Clockwise => {
                menu.up()
                    .expect("Menu navigation failed!")
            },
            RotDir::Anticlockwise => {
                menu.down()
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


        // Update inputs page
        if core::ptr::eq(menu.current_page(), &PAGE_LIST[2]) {
            let io10_state = io10.read() == Level::High;
            let io11_state = io11.read() == Level::High;
            let io12_state = io12.read() == Level::High;
            let io13_state = io13.read() == Level::High;

            if menu.state.io10_state ^ io10_state {
                // Input changed, update it

                menu.move_cursor(7, 1)      // Position of first X in panel
                        .expect("Moving cursor failed!");

                if io10_state {
                    menu.write_str(b"X")
                        .expect("Writing to display failed!")
                } else {
                    menu.write_str(b" ")
                        .expect("Writing to display failed!")
                }

                menu.state.io10_state = io10_state;
            }

            if menu.state.io11_state ^ io11_state {
                // Input changed, update it

                menu.move_cursor(8, 1)      // Position of second X in panel
                        .expect("Moving cursor failed!");

                if io11_state {
                    menu.write_str(b"X")
                        .expect("Writing to display failed!")
                } else {
                    menu.write_str(b" ")
                        .expect("Writing to display failed!")
                }

                menu.state.io11_state = io11_state;
            }

            
            if menu.state.io12_state ^ io12_state {
                // Input changed, update it

                menu.move_cursor(9, 1)      // Position of third X in panel
                        .expect("Moving cursor failed!");

                if io12_state {
                    menu.write_str(b"X")
                        .expect("Writing to display failed!")
                } else {
                    menu.write_str(b" ")
                        .expect("Writing to display failed!")
                }

                menu.state.io12_state = io12_state;
            }

            if menu.state.io13_state ^ io13_state {
                // Input changed, update it

                menu.move_cursor(10, 1)      // Position of fourth X in panel
                        .expect("Moving cursor failed!");

                if io13_state {
                    menu.write_str(b"X")
                        .expect("Writing to display failed!")
                } else {
                    menu.write_str(b" ")
                        .expect("Writing to display failed!")
                }

                menu.state.io13_state = io13_state;
            }
        }

        std::thread::sleep(
            Duration::from_micros(500)
        );
    }
}