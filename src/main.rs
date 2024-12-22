use gpio_cdev::{Chip, Line, LineHandle, LineRequestFlags, MultiLineHandle};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

const REGISTER_SELECT: u32 = 20;
const ENABLE: u32 = 21;
const D4: u32 = 25;
const D5: u32 = 8;
const D6: u32 = 7;
const D7: u32 = 1;

const INSTRUCTION_REGISTER: u8 = 0;
const DATA_REGISTER: u8 = 1;
const ENABLED: u8 = 0;
const DISABLED: u8 = 1;
const LOW: u8 = 0;

const STANDARD_PI_GPIO_DEVICE_PATH: &str = "/dev/gpiochip0";

fn instruction_register(line: &Line) -> Result<LineHandle, Box<dyn Error>> {
    let register_select = line.request(
        LineRequestFlags::OUTPUT,
        INSTRUCTION_REGISTER,
        "lcd_rs_register_select",
    )?;
    Ok(register_select)
}

fn data_register(line: &Line) -> Result<LineHandle, Box<dyn Error>> {
    let register_select = line.request(
        LineRequestFlags::OUTPUT,
        DATA_REGISTER,
        "lcd_rs_register_select",
    )?;
    Ok(register_select)
}

fn toggle(line: &LineHandle, duration: Duration) -> Result<(), Box<dyn Error>> {
    line.set_value(ENABLED)?;
    sleep(duration);
    line.set_value(DISABLED)?;
    Ok(())
}

fn send_4(values: &[u8; 4], data_handle: &MultiLineHandle) -> Result<(), Box<dyn Error>> {
    Ok(data_handle.set_values(values)?)
}

fn setup_lcd(
    register_select_line: &Line,
    data: &MultiLineHandle,
    enable: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    let _ = instruction_register(register_select_line)?;

    // Handle post-reset initialization into 4 bit mode
    data.set_values(&[0, 0, 1, 1])?; // Post RESET 'A' - Device thinks this is 0011 0000, same as 8 bit mode
    toggle(&enable, Duration::from_micros(4100))?; // Wait "more than 4.1 milliseconds"
    data.set_values(&[0, 0, 1, 1])?; // Post RESET 'B' - Device thinks this is 0011 0000, same as 8 bit mode
    toggle(&enable, Duration::from_micros(100))?; // Wait "more than 100 microseconds"
    data.set_values(&[0, 0, 1, 1])?; // Post RESET 'C' - Device thinks this is 0011 0000, same as 8 bit mode
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[0, 0, 1, 0])?; // In 8 bit mode this would need to be 0001, in either mode 0010 moves us into 4 bit mode...
    toggle(&enable, Duration::from_millis(40))?;

    // Now do the actual post-reset 4-bit mode setup (always HI then LO as this is big-endian)
    data.set_values(&[0, 0, 1, 0])?; // Function Set  ... DATA LENGTH = 4 bits, LINES = 2, FONT = 5x8
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[1, 0, 0, 0])?;
    toggle(&enable, Duration::from_millis(40))?;

    data.set_values(&[0, 0, 0, 0])?; // Display On/off .. DISPLAY ON, CURSOR OFF, BLINK OFF
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[1, 1, 0, 0])?;
    toggle(&enable, Duration::from_millis(40))?;

    data.set_values(&[0, 0, 0, 0])?; // Clear display
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[0, 0, 0, 1])?;
    toggle(&enable, Duration::from_millis(40))?;

    data.set_values(&[0, 0, 0, 0])?; // Set cursor to home position
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[0, 0, 1, 0])?;
    toggle(&enable, Duration::from_millis(40))?;

    data.set_values(&[0, 0, 0, 0])?; // Entry Mode ... INCREMENT, SHIFT = OFF (same as after RESET, could omit this)
    toggle(&enable, Duration::from_millis(40))?;
    data.set_values(&[0, 1, 1, 0])?;
    toggle(&enable, Duration::from_millis(40))?;

    Ok(())
}

fn send_char(
    character: char,
    data_handle: &MultiLineHandle,
    enable_handle: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    if character.is_ascii() {
        let ascii = character as u8;

        // Not very classy, but it's easy (for me) to understand and it works...
        let high: [u8; 4] = [
            if ascii & 0b10000000 > 0 { 1 } else { 0 },
            if ascii & 0b01000000 > 0 { 1 } else { 0 },
            if ascii & 0b00100000 > 0 { 1 } else { 0 },
            if ascii & 0b00010000 > 0 { 1 } else { 0 },
        ];

        let low: [u8; 4] = [
            if ascii & 0b00001000 > 0 { 1 } else { 0 },
            if ascii & 0b00000100 > 0 { 1 } else { 0 },
            if ascii & 0b00000010 > 0 { 1 } else { 0 },
            if ascii & 0b00000001 > 0 { 1 } else { 0 },
        ];

        send_4(&high, data_handle)?;
        toggle(enable_handle, Duration::from_millis(40))?;
        send_4(&low, data_handle)?;
        toggle(enable_handle, Duration::from_millis(40))?;
    }

    Ok(())
}

fn send_text_to_lcd(
    text: &str,
    register_select_line: &Line,
    data_handle: &MultiLineHandle,
    enable_handle: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    let _ = data_register(register_select_line)?;

    for c in text.chars() {
        send_char(c, data_handle, enable_handle)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Get a handle to the GPIO device...");
    let mut chip = Chip::new(STANDARD_PI_GPIO_DEVICE_PATH)?;

    let register_select_line = chip.get_line(REGISTER_SELECT)?;
    let enable_line = chip.get_line(ENABLE)?;
    let data_lines = chip.get_lines(&[D7, D6, D5, D4])?;

    println!("Register the output lines...");
    // Note - "consumer names" will be visible via the gpuinfo cli tool
    let data_handle = data_lines.request(
        LineRequestFlags::OUTPUT,
        &[LOW, LOW, LOW, LOW],
        "lcd_rs_data",
    )?;
    let enable_handle = enable_line.request(LineRequestFlags::OUTPUT, DISABLED, "lcd_rs_enable")?;

    println!("Setup the LCD");
    setup_lcd(&register_select_line, &data_handle, &enable_handle)?;

    println!("Start the text output");
    send_text_to_lcd(
        "Hello, World!",
        &register_select_line,
        &data_handle,
        &enable_handle,
    )?;

    println!("Done, shutdown.");
    Ok(())
}