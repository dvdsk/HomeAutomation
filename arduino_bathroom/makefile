CXX = avr-g++
CXXFLAGS= -Wall -pedantic -std=c++14 \
				  -O3 \
					-DF_CPU=16000000UL \
					-mmcu=atmega328p \

#ARDUINO
SPI = /usr/share/arduino/libraries/SPI/SPI.h
#RF24

LDFLAGS = -I /usr/share/arduino
#LDLIBS = -lssl -lcrypto

PORT = /dev/ttyUSB0

ODIR = .BUILD
OBJS = $(addprefix $(ODIR)/,arduino_bathroom.o humiditySensor.o fastSensors.o encodingScheme.o)

help:
	@echo 'Help details:'
	@echo 'hex: compile hex file'
	@echo 'flash: install hex file'
	@echo 'program: compile hex and install'

hex: $(OBJS)
	$(CXX) $(CXXFLAGS) -c -o led.o led.c
	$(CXX) -mmcu=atmega328p led.o -o temp

$(ODIR)/arduino_bathroom.o: arduino_bathroom.cpp fastSensors.h humiditySensor.h \
	encodingScheme.h config.h
	$(CXX) $(CXXFLAGS) $(LDFLAGS) -o$@ -c arduino_bathroom.cpp

$(ODIR)/humiditySensor.o: humiditySensor.cpp humiditySensor.h config.h
	$(CXX) $(CXXFLAGS) -o$@ -c humiditySensor.cpp
$(ODIR)/fastSensors.o: fastSensors.cpp fastSensors.h config.h encodingScheme.h compression.h
	$(CXX) $(CXXFLAGS) -o$@ -c fastSensors.cpp	
$(ODIR)/encodingScheme.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
  dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainData.cpp

flash:
	avr-objcopy -O ihex -R .eeprom temp temp.hex
	avrdude /usr/share/arduino/hardware/tools/avrdude.conf -c arduino -p atmega328p -P $(PORT) -b 57600 -U flash:w:temp.hex

.PHONY : clean
clean:
	rm temp $(OBJS)
