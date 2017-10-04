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

#if (ARDUINO >= 100)
 #include "Arduino.h"
#else
 #include "WProgram.h"
#endif
#include "Wire.h"

//#define SHT31_DEFAULT_ADDR    0x44
/*#define SHT31_MEAS_HIGHREP_STRETCH 0x2C06*/
/*#define SHT31_MEAS_MEDREP_STRETCH  0x2C0D*/
/*#define SHT31_MEAS_LOWREP_STRETCH  0x2C10*/
/*#define SHT31_MEAS_HIGHREP         0x2400*/
/*#define SHT31_MEAS_MEDREP          0x240B*/
/*#define SHT31_MEAS_LOWREP          0x2416*/
/*#define SHT31_READSTATUS           0xF32D*/
/*#define SHT31_CLEARSTATUS          0x3041*/
/*#define SHT31_SOFTRESET            0x30A2*/
/*#define SHT31_HEATEREN             0x306D*/
/*#define SHT31_HEATERDIS            0x3066*/

constexpr uint8_t addr = 0x44;
constexpr uint16_t measureHighRes = 0x2400;
constexpr uint16_t softReset = 0x30A2;

namespace TempHum{
	void begin();
	void reset();
	void request();
	bool readyToRead();
	void read(uint16_t &temp, uint16_t &hum);
	void setToNan(uint16_t &temp, uint16_t &hum);
	void writeCommand(uint16_t cmd);
	uint8_t crc8(const uint8_t *data, uint8_t len);
}
