use pwm_pca9685::Channel;
use systep::ControllerData;

/* Stepper Controller */
    /// Pins for the step control of the stepper motors
    pub const RDX_PIN_STEP : [u8; 4] = [
        24, 25, 5, 6
    ];

    /// Pins for the direction control of the stepper motors
    pub const RDX_PIN_DIR : [u8; 4] = [
        23, 22, 12, 20
    ];

    /// Pin number of the enable pin for all the stepper controllers
    pub const RDX_PIN_SC_EN : u8 = 26;      // IS USUALLY 16, CURRENTLY WIRED DIFFERENTLY

    /// RDX Stepper controller data, all drivers are identical
    pub const RDX_DATA_SC : ControllerData = ControllerData { 
        max_freq: 50_000.0
    };
/**/

/* Servos */
    pub const RDX_CHANNEL_SERVO : [Channel; 8] = [
        Channel::C0, Channel::C1, Channel::C2, Channel::C3, Channel::C4, Channel::C5, Channel::C6, Channel::C7
    ];
/* */

/* DC-Drivers */
    pub const RDX_CHANNEL_DC : [Channel; 6] = [ 
        Channel::C8, Channel::C9, Channel::C10, Channel::C11, Channel::C12, Channel::C13
    ];
    pub const RDX_CHANNEL_FAN : [Channel; 2] = [ Channel::C14, Channel::C15 ];
/**/

/* GPIO-Connectors */
    pub const RDX_PIN_IO1 : [u8; 4] = [
        4, 27, 21, 13
    ];

    pub const RDX_PIN_IO2 : [u8; 4] = [
        10, 9, 11, 8
    ];

    pub const RDX_PIN_MISC : [u8; 4] = [
        24, 25, 5, 6
    ];
/**/

// I2C
pub const RDX_I2C_ADDR_LCD : u8 = 0x27;
pub const RDX_I2C_ADDR_PCA9685 : u8 = 0x40;

// Rotary encoder
pub const RDX_PIN_ROT_DT : u8 = 18;
pub const RDX_PIN_ROT_CL : u8 = 17;
pub const RDX_PIN_ROT_SW : u8 = 19;