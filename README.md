# LCD Rust

A simple Rust example for driving a 16x2 LCD controlled by a HD44780U or 
equivalent from a Raspberry Pi 5 using the "proper" ioctl based stuff.

There's a Python (*not* MicroPython) example as well for comparison. The
Python script does not use the Busy signal so it's relatively slow but
my aim is to make this Rust example do that and we'll see how it 
improves things.

## Usage notes

This is all being done & tested under 64 bit **Ubuntu** (*not* Raspbian) but 
I'd expect it all to work there equally well and I'll add a note once I've 
tested it on that platform as well.

This is all on the Pi 5, so absolutely no guarantees it will work on other hardware.

## Status etc.

Not even started on the Rust side! 

The [python script](lcd.py) works ok (albeit slowly). It requires the Python [lgpio library](https://pypi.org/project/lgpio/):
```bash
sudo apt install python3-lgpio
```

## Useful Resources

 * [A MicroPython script that I based my Python script on](https://how2electronics.com/interfacing-16x2-lcd-display-with-raspberry-pi-pico/)
 * [Ubuntu GPIO usage docs](https://ubuntu.com/tutorials/gpio-on-raspberry-pi#1-overview)
 * [Python LGPIO library docs](http://abyz.me.uk/lg/py_lgpio.html#gpio_claim_output)
 * [LGPIO Docs on PyPi](https://pypi.org/project/lgpio/)
 * [HD44780 datasheet](https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)
 * [A blog on HD44780 usage](https://www.gibbard.me/hd44780_lcd_screen/)
 * [The Pi 40 pin GPIO header pinout](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#gpio)
 * [About the Linux GPIO usage changes](https://waldorf.waveform.org.uk/2021/the-pins-they-are-a-changin.html)
 * [Ben Eater's video on connecting an LCD to his homebrew 6502 computer](https://www.youtube.com/watch?v=FY3zTUaykVo)
