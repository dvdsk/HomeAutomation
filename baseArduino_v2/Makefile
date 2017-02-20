#Arduino Make file. Refer to https://github.com/sudar/Arduino-Makefile

# --- mega2560 ide 1.0
BOARD_TAG    = mega2560
ARDUINO_PORT = /dev/ttyUSB0
ARDUINO_LIBS = SHT1x Wire SPI RF24-master

AVR_TOOLS_DIR = /usr
MONITOR_BAUDRATE = 9600

CXXFLAGS += -Wall -Wextra -std=gnu++11 -DDEBUG -O3
include /usr/share/arduino/Arduino.mk

