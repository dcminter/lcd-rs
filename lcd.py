# Drive a hello world message *from Ubuntu* to a 16x02 LCD controlled by
# a Hitachi 44780 or compatible
#
# Based on the following docs:
#
# * A MicroPython script that I based this on: https://how2electronics.com/interfacing-16x2-lcd-display-with-raspberry-pi-pico/
# * Ubuntu GPIO usage docs: https://ubuntu.com/tutorials/gpio-on-raspberry-pi#1-overview
# * Python LGPIO library docs: http://abyz.me.uk/lg/py_lgpio.html#gpio_claim_output
# * HD44780 datasheet: https://www.sparkfun.com/datasheets/LCD/HD44780.pdf
# * A blog on HD44780 usage: https://www.gibbard.me/hd44780_lcd_screen/
#
import time
import lgpio

# Define pins
REGISTER_SELECT = 20
ENABLE = 21
D4 = 25
D5 = 8
D6 = 7
D7 = 1

INSTRUCTION_REGISTER = 0
DATA_REGISTER = 1

print("Getting GPIO handle")
handle = lgpio.gpiochip_open(0)
if handle >= 0:
   print("GPIO Handle OK")
else:
   print("Uh oh, GPIO Handle NOT OK")

print("Claiming pins")
lgpio.group_claim_output(handle, [ENABLE, REGISTER_SELECT, D4, D5, D6, D7])

def sleep_ms(duration):
    time.sleep(duration * 0.001)

def pulse_enable():
    lgpio.gpio_write(handle, ENABLE, 1)
    sleep_ms(40)
    lgpio.gpio_write(handle, ENABLE, 0)
    sleep_ms(40)

def send_4(BinNum):
    lgpio.gpio_write(handle, D4, (BinNum & 0b00000001) >>0) # This right-shift approach seems clumsy; other options?
    lgpio.gpio_write(handle, D5, (BinNum & 0b00000010) >>1)
    lgpio.gpio_write(handle, D6, (BinNum & 0b00000100) >>2)
    lgpio.gpio_write(handle, D7, (BinNum & 0b00001000) >>3)
    pulse_enable()

def send_8(BinNum):
    lgpio.gpio_write(handle, D4, (BinNum & 0b00010000) >>4)
    lgpio.gpio_write(handle, D5, (BinNum & 0b00100000) >>5)
    lgpio.gpio_write(handle, D6, (BinNum & 0b01000000) >>6)
    lgpio.gpio_write(handle, D7, (BinNum & 0b10000000) >>7)
    pulse_enable()
    lgpio.gpio_write(handle, D4, (BinNum & 0b00000001) >>0)
    lgpio.gpio_write(handle, D5, (BinNum & 0b00000010) >>1)
    lgpio.gpio_write(handle, D6, (BinNum & 0b00000100) >>2)
    lgpio.gpio_write(handle, D7, (BinNum & 0b00001000) >>3)
    pulse_enable()

def clear():
    lgpio.gpio_write(handle, REGISTER_SELECT, INSTRUCTION_REGISTER)
    send_8(0b00000000)

def setup_lcd():
    # 4 Bit initialization, see page 46 of the data-sheet!
    print("Setup LCD")
    sleep_ms(15) # Probably not necessary as power will have been supplied for way longer, than 15ms, but let's be careful!
    lgpio.gpio_write(handle, REGISTER_SELECT, INSTRUCTION_REGISTER) # Put into command mode

    # Handle post-reset initialization into 4 bit mode
    send_4(0b0011) # Post RESET 'A' - Device thinks this is 0011 0000, same as 8 bit more
    sleep_ms(5)    # Wait "more than 4.1 milliseconds"
    send_4(0b0011) # Post RESET 'B' - Device thinks this is 0011 0000, same as 8 bit more
    sleep_ms(0.1)  # Wait "more than 100 microseconds"
    send_4(0b0011) # Post RESET 'C' - Device thinks this is 0011 0000, same as 8 bit more
    send_4(0b0010) # In 8 bit mode this would need to be 0001, in either mode 0010 moves us into 4 bit mode...

    # Expecting 4 bit mode from here on; set the display up as desired:
    send_8(0b00101000) # Function Set  ... DATA LENGTH = 4 bits, LINES = 2, FONT = 5x8
    send_8(0b00001100) # Display On/off .. DISPLAY ON, CURSOR OFF, BLINK OFF
    send_8(0b00000001) # Clear display
    send_8(0b00000010) # Set to home position
    send_8(0b00000110) # Entry Mode ... INCREMENT, SHIFT = OFF (same as after RESET, could omit this)

    lgpio.gpio_write(handle, REGISTER_SELECT, DATA_REGISTER) # Change to normal text-writing mode...

setup_lcd()
for x in 'Hello World!':
    send_8(ord(x))
    print(f"Printing '{x}'")

# Not really necessary; I used this when I was testing that the lines were getting set
# as-expected via some LEDs instead of the LCD itself - at the end, depending on what
# data is written the RS line and potentially some data lines are left high, which
# seemed kinda messy. But there's no particular reason to do this, so commented
# out for now!
#clear()

print("Releasing GPIO handle")
lgpio.gpiochip_close(handle)
print("Done.")