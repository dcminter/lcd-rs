use gpio_cdev::{Chip, Line, LineHandle, LineRequestFlags, MultiLineHandle};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

// Define GPIO pins to use
const REGISTER_SELECT: u32 = 20; // Pin 38 on Pi
const ENABLE: u32 = 21; // Pin 40 on Pi
const D4: u32 = 25; // Pin 22 on Pi
const D5: u32 = 8; // Pin 24 on Pi
const D6: u32 = 7; // Pin 26 on Pi
const D7: u32 = 1; // Pin 28 on Pi
const READ_WRITE_SELECT: u32 = 12; // Pin 32 on Pi

// Define meaning of signals on pins
const INSTRUCTION_REGISTER: u8 = 0;
const DATA_REGISTER: u8 = 1;
const ENABLED: u8 = 0;
const DISABLED: u8 = 1;
const LOW: u8 = 0;
const READ: u8 = 1;
const WRITE: u8 = 0;

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

fn send_4<F: FnOnce() -> Result<(), Box<dyn Error>>>(
    values: &[u8; 4],
    data: &(Line, MultiLineHandle),
    read_write_handle: &LineHandle,
    toggler: F,
) -> Result<(), Box<dyn Error>> {
    // D7 is special... because in INPUT mode it's the busy flag.
    {
        let _ = data
            .0
            .request(LineRequestFlags::OUTPUT, values[0], "lcd_rs_data_d7")?;
        data.1.set_values(&values[1..])?;
        toggler()
    }?;

    // Flip RW flag into READ mode and read D7 until we get a non-busy state
    read_write_handle.set_value(READ)?;
    {
        let busy = data
            .0
            .request(LineRequestFlags::INPUT, LOW, "lcd_rs_data_busy")?;
        while busy.get_value()? > 0 {
            println!("Busy");
        }
    }

    // Put the RW flag back into WRITE mode
    println!("Not busy");
    read_write_handle.set_value(WRITE)?;
    // But we're still in WRITE mode...

    Ok(())
}

fn setup_lcd(
    read_write_handle: &LineHandle,
    register_select_line: &Line,
    data: &(Line, MultiLineHandle),
    enable: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    let _ = instruction_register(register_select_line)?;

    // Mostly we're being conservative and using this 40ms toggle (i.e. set ENABLE pin high for 40ms). Elsewhere during the reset I'm using the "more than" delays specified by the flow diagram in the HD44780U datasheet
    let toggle_40ms = || toggle(&enable, Duration::from_millis(40));

    ////
    // Handle post-reset initialization into 4 bit mode

    // Post RESET 'A' - Device thinks this is 0011 0000, same as 8 bit mode. Wait "more than 4.1 milliseconds"
    send_4(&[0, 0, 1, 1], data, read_write_handle, || {
        toggle(&enable, Duration::from_micros(4100))
    })?;

    // Post RESET 'B' - Device thinks this is 0011 0000, same as 8 bit mode. Wait "more than 100 microseconds"
    send_4(&[0, 0, 1, 1], data, read_write_handle, || {
        toggle(&enable, Duration::from_micros(100))
    })?;

    // Post RESET 'C' - Device thinks this is 0011 0000, same as 8 bit mode
    send_4(&[0, 0, 1, 1], data, read_write_handle, toggle_40ms)?;

    // In 8 bit mode this would need to be 0001, in either mode 0010 moves us into 4 bit mode...
    send_4(&[0, 0, 1, 0], data, read_write_handle, toggle_40ms)?;

    ////
    // Now do the actual post-reset 4-bit mode setup (always HI then LO as this is big-endian)

    // Function Set  ... DATA LENGTH = 4 bits, LINES = 2, FONT = 5x8
    send_4(&[0, 0, 1, 0], data, read_write_handle, toggle_40ms)?;
    send_4(&[1, 0, 0, 0], data, read_write_handle, toggle_40ms)?;

    // Display On/off .. DISPLAY ON, CURSOR OFF, BLINK OFF
    send_4(&[0, 0, 0, 0], data, read_write_handle, toggle_40ms)?;
    send_4(&[1, 1, 0, 0], data, read_write_handle, toggle_40ms)?;

    // Clear display
    send_4(&[0, 0, 0, 0], data, read_write_handle, toggle_40ms)?;
    send_4(&[0, 0, 0, 1], data, read_write_handle, toggle_40ms)?;

    // Set cursor to home position
    send_4(&[0, 0, 0, 0], data, read_write_handle, toggle_40ms)?;
    send_4(&[0, 0, 1, 0], data, read_write_handle, toggle_40ms)?;

    // Entry Mode ... INCREMENT, SHIFT = OFF (same as after RESET, could omit this)
    send_4(&[0, 0, 0, 0], data, read_write_handle, toggle_40ms)?;
    send_4(&[0, 1, 1, 0], data, read_write_handle, toggle_40ms)?;

    Ok(())
}

fn send_char(
    character: char,
    data: &(Line, MultiLineHandle),
    read_write_handle: &LineHandle,
    enable_handle: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    if character.is_ascii() {
        let ascii = character as u8;

        // Not very classy, but it's easy (for me) to understand and it works... sticking a fn
        // in doesn't make it any clearer, so I'm keeping like this until shown a better way!
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

        let toggler = || toggle(enable_handle, Duration::from_millis(40));

        send_4(&high, data, read_write_handle, toggler)?;
        send_4(&low, data, read_write_handle, toggler)?;
    }

    Ok(())
}

fn send_text_to_lcd(
    text: &str,
    read_write_handle: &LineHandle,
    register_select_line: &Line,
    data: &(Line, MultiLineHandle),
    enable_handle: &LineHandle,
) -> Result<(), Box<dyn Error>> {
    let _ = data_register(register_select_line)?;

    for c in text.chars() {
        send_char(c, data, read_write_handle, enable_handle)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Get a handle to the GPIO device...");
    let mut chip = Chip::new(STANDARD_PI_GPIO_DEVICE_PATH)?;

    let register_select_line = chip.get_line(REGISTER_SELECT)?;
    let d7_data_line = chip.get_line(D7)?;
    let data_lines = chip.get_lines(&[D6, D5, D4])?;
    let enable_line = chip.get_line(ENABLE)?;
    let read_write_line = chip.get_line(READ_WRITE_SELECT)?;

    println!("Register the output lines...");
    // Note - "consumer names" will be visible via the gpuinfo cli tool
    let data_handle =
        data_lines.request(LineRequestFlags::OUTPUT, &[LOW, LOW, LOW], "lcd_rs_data")?;

    let data = (d7_data_line, data_handle);

    let enable_handle = enable_line.request(LineRequestFlags::OUTPUT, DISABLED, "lcd_rs_enable")?;
    let read_write_handle =
        read_write_line.request(LineRequestFlags::OUTPUT, WRITE, "lcd_rs_read_write")?;
    // Verified that the board still works ok with the RW line held low via the GPIO pin

    println!("Setup the LCD");
    setup_lcd(
        &read_write_handle,
        &register_select_line,
        &data,
        &enable_handle,
    )?;

    println!("Start the text output");
    send_text_to_lcd(
        "Hello, World!",
        &read_write_handle,
        &register_select_line,
        &data,
        &enable_handle,
    )?;

    println!("Done, shutdown.");
    Ok(())
}
