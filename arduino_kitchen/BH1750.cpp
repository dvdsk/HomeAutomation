/*

This is a library for the BH1750FVI Digital Light Sensor
breakout board.

The board uses I2C for communication. 2 pins are required to
interface to the device.


Written by Christopher Laws, March, 2013.

*/

#include "BH1750.h"
#include <util/delay.h>


BH1750::BH1750(bool highAddr, uint8_t mode_) {
	if(highAddr)
		addr = BH1750_I2CADDR_L;
	else
		addr = BH1750_I2CADDR_H;
	mode = mode_;
}

void BH1750::begin(){
	 // will only call if it hasn't been called yet
	if (TWCR == 0){ // do this check so that Wire only gets initialized once
		 Serial.println("Beginning wire library...");
		 Wire.begin();
	}
	configure(mode);
}

void BH1750::configure(uint8_t mode) {

    switch (mode) {
        case BH1750_CONTINUOUS_HIGH_RES_MODE:
        case BH1750_CONTINUOUS_HIGH_RES_MODE_2:
        case BH1750_CONTINUOUS_LOW_RES_MODE:
        case BH1750_ONE_TIME_HIGH_RES_MODE:
        case BH1750_ONE_TIME_HIGH_RES_MODE_2:
        case BH1750_ONE_TIME_LOW_RES_MODE:
            // apply a valid mode change
            write8(mode);
            _delay_ms(10);
            break;
        default:
            // Invalid measurement mode
            #if BH1750_DEBUG == 1
            Serial.println("Invalid measurement mode");
            #endif
            break;
    }
}


uint16_t BH1750::readLightLevel(void) {

  uint16_t level;

  Wire.beginTransmission(addr);
  Wire.requestFrom(addr, (uint8_t)2);
#if (ARDUINO >= 100)
  level = Wire.read();
  level <<= 8;
  level |= Wire.read();
#else
  level = Wire.receive();
  level <<= 8;
  level |= Wire.receive();
#endif
  Wire.endTransmission();

#if BH1750_DEBUG == 1
  Serial.print("Raw light level: ");
  Serial.println(level);
#endif

  level = level/1.2; // convert to lux

#if BH1750_DEBUG == 1
  Serial.print("Light level: ");
  Serial.println(level);
#endif
  return level;
}



/*********************************************************************/


void BH1750::write8(uint8_t d) {
  Wire.beginTransmission(addr);
#if (ARDUINO >= 100)
  Wire.write(d);
#else
  Wire.send(d);
#endif
  Wire.endTransmission();
}

