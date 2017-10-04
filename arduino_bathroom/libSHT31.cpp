/*************************************************** 
  This is an adaptation to the lib made by adafruit. It was changed to a 
	request then check if rdy then read style at the user level. This allowes
	the user to do other stuff in between. 

	origional text by adafruit:
		This is a library for the SHT31 Digital Humidity & Temp Sensor

		Designed specifically to work with the SHT31 Digital sensor from Adafruit
		----> https://www.adafruit.com/products/2857

		These sensors use I2C to communicate, 2 pins are required to interface
		Adafruit invests time and resources providing this open source code, 
		please support Adafruit and open-source hardware by purchasing 
		products from Adafruit!

		Written by Limor Fried/Ladyada for Adafruit Industries.  
		BSD license, all text above must be included in any redistribution
 ****************************************************/
#include "libSHT31.h"

void TempHum::begin() {
	if (TWCR == 0){ // do this check so that Wire only gets initialized once
		Wire.begin(); 
		//options for speed: 400000 (fast mode), 100000 (standard mode), 
		//1000000 (fast mode plus) and 3400000 (high speed mode)
		//Wire.setClock(1000000); 
	}
	reset();
  return true;
}

void TempHum::reset(){
  writeCommand(softReset);
	delay(10);
  //return (readStatus() == 0x40);
  return true;
}

void TempHum::request(){
  writeCommand(measureHighRes);
}

bool TempHum::readyToRead(){
	Wire.requestFrom(addr, (uint8_t)6, (uint8_t)false);
	if (Wire.available() != 6) 
		return false;
	else{
		return true;
	}
}

void TempHum::read(uint16_t &temp, uint16_t &hum){
  uint8_t readbuffer[6];

	for (uint8_t i=0; i<6; i++) {
		readbuffer[i] = Wire.read();
		//  Serial.print("0x"); Serial.println(readbuffer[i], HEX); //debug
	}
	uint16_t ST, SRH;
	ST = readbuffer[0];
	ST <<= 8;
	ST |= readbuffer[1];

	if (readbuffer[2] != crc8(readbuffer, 2)){setToNan(temp, hum); return;}

	SRH = readbuffer[3];
	SRH <<= 8;
	SRH |= readbuffer[4];

	if (readbuffer[5] != crc8(readbuffer+3, 2)){setToNan(temp, hum); return;}

	// Serial.print("ST = "); Serial.println(ST);
	double stemp = ST;
	stemp *= 175;
	stemp /= 0xffff;
	stemp = -45 + stemp;
	temp = (uint16_t)((stemp+10)*10);

	//  Serial.print("SRH = "); Serial.println(SRH);
	double shum = SRH;
	shum *= 100;
	shum /= 0xFFFF;
	hum = (uint16_t)(shum*10);
}

void TempHum::setToNan(uint16_t &temp, uint16_t &hum){
	temp = -11;
	hum = 1001;
}

void TempHum::writeCommand(uint16_t cmd) {
  Wire.beginTransmission(addr);
  Wire.write(cmd >> 8);
  Wire.write(cmd & 0xFF);
  Wire.endTransmission(false);  
}

uint8_t TempHum::crc8(const uint8_t *data, uint8_t len)
{
/*
*
 * CRC-8 formula from page 14 of SHT spec pdf
 *
 * Test data 0xBE, 0xEF should yield 0x92
 *
 * Initialization data 0xFF
 * Polynomial 0x31 (x8 + x5 +x4 +1)
 * Final XOR 0x00
 */

  const uint8_t POLYNOMIAL(0x31);
  uint8_t crc(0xFF);
  
  for ( int j = len; j; --j ) {
      crc ^= *data++;

      for ( int i = 8; i; --i ) {
	crc = ( crc & 0x80 )
	  ? (crc << 1) ^ POLYNOMIAL
	  : (crc << 1);
      }
  }
  return crc;
}
