#ifndef CONFIG
#define CONFIG

//the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways
typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

//TODO can be destroyed?
namespace fDat {
	constexpr int PIRS = 0;
	constexpr int LIGHT_BED = 1;
}

//TODO can be destroyed?
namespace sdat {
	constexpr int CO2 = 0;
}

namespace config {
	constexpr int CALIBRATION_TIME = 1000; //milliseconds
	constexpr int READSPEED = 1; //millisec
	constexpr int RESETSPEED = 500;
}

//pins
namespace pin {
	constexpr int TERM_DATA = 24; //PA2 (hard coded register banks)
	constexpr int TERM_CLOCK = 22; //PA0 (hard coded register banks)

	constexpr int LIGHT_BED = 0; //anolog in
	constexpr int PIR_BED_NORTH = 26; //PA4
	constexpr int PIR_BED_SOUTH = 28; //PA6

	constexpr int RADIO_CE = 7;
	constexpr int RADIO_CS = 9;
}

namespace headers {
	constexpr uint8_t SETUP_DONE = 200;
	constexpr uint8_t STARTUP_DONE = 201;
	constexpr uint8_t FAST_UPDATE = 255;
	constexpr uint8_t SLOW_UPDATE = 26;
}

namespace radioRQ {
	constexpr uint8_t NODE1_FAST_UPDATE = 1;
	constexpr uint8_t NODE2_FAST_UPDATE = 2;

	constexpr uint8_t NODE1_SLOW_UPDATE = 3;
	constexpr uint8_t NODE2_SLOW_UPDATE = 4;

	constexpr uint8_t NODE1_RESEND_SLOW = 5;
	constexpr uint8_t NODE2_RESEND_SLOW = 6;
}

//these are indexes in a uint16_t array
namespace Idx {
	//fast package
	constexpr int UPDATED = 0;								//1 byte, not send
	constexpr int CO2 = 1;										//2 bytes

	constexpr int TEMPERATURE_BED = 2;				//2 bytes
	constexpr int HUMIDITY_BED = 3;						//2 bytes

	constexpr int TEMPERATURE_DOOR = 4;				//2 bytes
	constexpr int HUMIDITY_DOOR = 5;					//2 bytes

	constexpr int TEMPERATURE_BATHROOM = 6;		//2 bytes
	constexpr int HUMIDITY_BATHROOM = 7;			//2 bytes

	//slow package
	constexpr int PIRS = 0;										//1 byte
	constexpr int PIRS_UPDATED = 1;						//1 byte
	
	constexpr int LIGHT_BED = 2;							//2 bytes
	constexpr int LIGHT_DOOR = 3;							//2 bytes
	constexpr int LIGHT_KITCHEN = 4;					//2 bytes
}

//needed constants
constexpr uint8_t RADIO_ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes 
constexpr byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; //TODO change to uint8_t if possible

/*Encoding Scheme:
FastData
	pirs: N-bits
	pirs_updated: N-bits

	light: 10 bits, storing raw anolog read value. 3x

	Order as indicated in the namespace Idx

SlowData
	Temp: 9 bits        [storing -10.0 to 40.0 degrees, in values 0 to 500,
		                  values 501 means lower then -10.0 and 502 higher then 40.0]]
	Humidity: 10 bits   [storing 0.0 to 100.0 percent, in values 0 to 1000]
	Co2: 13 bits        [storing 0 to 6000ppm, in values 0 to 6000]

	Order as indicated in the namespace Idx
*/


constexpr uint8_t FASTDATA_SIZE = 4;
constexpr uint8_t SLOWDATA_SIZE = 7;
constexpr uint16_t SLOWDATA_COMPLETE = 0 | (1 << Idx::TEMPERATURE_BED) 
																				 | (1 << Idx::HUMIDITY_BED)
																				 | (1 << Idx::CO2);


#endif

