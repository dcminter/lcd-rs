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

## Choices

I'm going to use crate [gpio-cdev](https://crates.io/crates/gpio-cdev) for the following reasons:

* Fairly current, with recent commits
* Close to the current base Linux mechanisms
* From the Rust-Embedded working group
    * [Working Group](https://www.rust-lang.org/governance/wgs/embedded)
    * [Working Group Repos](https://github.com/rust-embedded)
* Looks like it gets enough eyes for anything nefarious to be spotted

## Status...

This isn't very elegant, but it works (on my machine, obv.)

The Read/Write pin on the display is still wired to ground, so I can't use the busy signal yet - instead I'm just 
using short delays. they're roughly based on the maximum delay (37ms) given for some of the LCD setup instructions
when the instruction register is selected (or where given, the defined delays for the setup commands). Once I wire in 
the Read/Write pin, though, I can try using the edge-detection features to drive things as fast as the driver chip
allows.

### Device names

[Apparently](https://raspberrypi.stackexchange.com/questions/148477/how-to-determine-the-correct-gpio-chip-for-libgpiod) the device name for the GPIO chip on the Pi 5 is `gpiochip4` whereas on the Pi 4 it was
`gpiochip0` - however when I list gpio device files ...
```text
crw-rw---- 1 root gpio 254,  0 Dec 21 20:21 /dev/gpiochip0
crw-rw---- 1 root gpio 254, 10 Dec 21 20:21 /dev/gpiochip10
crw-rw---- 1 root gpio 254, 11 Dec 21 20:21 /dev/gpiochip11
crw-rw---- 1 root gpio 254, 12 Dec 21 20:21 /dev/gpiochip12
crw-rw---- 1 root gpio 254, 13 Dec 21 20:21 /dev/gpiochip13
lrwxrwxrwx 1 root root       9 Dec 21 20:21 /dev/gpiochip4 -> gpiochip0
crw-rw---- 1 root gpio 234,  0 Dec 21 20:21 /dev/gpiomem0
crw-rw---- 1 root gpio 238,  0 Dec 21 20:21 /dev/gpiomem1
crw-rw---- 1 root gpio 237,  0 Dec 21 20:21 /dev/gpiomem2
crw-rw---- 1 root gpio 236,  0 Dec 21 20:21 /dev/gpiomem3
crw-rw---- 1 root gpio 235,  0 Dec 21 20:21 /dev/gpiomem4
```
... I see that `gpiochip4` is now symlinked to `gpiochip0` - so in the hope of getting some
backward compatibility I'll use the `gpiochip0` name for now. Note that this is on 
Ubuntu, so it might not be true/usable on Raspbian; I'll make it configurable at some
point though.

### Consumer names

I'm a little hazy about exactly what the "consumer" name is - I probably need to pore over the Linux docs a bit more 
closely, but I get the impresison that it's purely informational.

I configured GPIO pin 1 as one of the data lines (D7) and named that group's consumer as `lcd_rs_data` and that's 
visible via the gpioinfo command while the program is running:
```bash
$ gpioinfo
gpiochip0 - 54 lines:
	line   0:     "ID_SDA"       unused   input  active-high 
	line   1:     "ID_SCL" "lcd_rs_data" output active-high [used]
...
etc.
```

## Python etc.

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
 * [Documentation for the Rust gpio-cdev library](https://docs.rs/gpio-cdev/latest/gpio_cdev/index.html)
 * [The Linux Kernel's GPIO API](https://docs.kernel.org/driver-api/gpio/)
 * [The Linux libgpiod library](https://git.kernel.org/pub/scm/libs/libgpiod/libgpiod.git/about/)