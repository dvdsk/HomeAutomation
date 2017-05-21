#ifndef CONFIG
#define CONFIG

//note also change this in config of decoder class (USED FOR DECODING/ENCODING AIRPRESSURE)
constexpr uint32_t MINIMUM_MEASURABLE_PRESSURE = 93600; //Pa

//the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways
typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

namespace config {
	constexpr int CALIBRATION_TIME = 2000; //milliseconds
	constexpr int READSPEED = 1; //millisec
	constexpr int RESETSPEED = 500;
}

//pins
namespace pin {
	constexpr int TERM_DATA = 24; //PA2 (hard coded register banks)
	constexpr int TERM_CLOCK = 22; //PA0 (hard coded register banks)

	constexpr int LIGHT_BED = 6; //anolog in
	constexpr int PIR_BED_NORTH = 26; //PA4
	constexpr int PIR_BED_SOUTH = 28; //PA6

	constexpr int RADIO_CE = 48;
	constexpr int RADIO_CS = 49;
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
	//slow package
	constexpr int UPDATED = 0;								//1 byte, not send
	constexpr int CO2 = 1;										//2 bytes

	constexpr int TEMPERATURE_BED = 2;				//2 bytes
	constexpr int HUMIDITY_BED = 3;						//2 bytes

	constexpr int TEMPERATURE_DOOR = 4;				//2 bytes
	constexpr int HUMIDITY_DOOR = 5;					//2 bytes

	constexpr int TEMPERATURE_BATHROOM = 6;		//2 bytes
	constexpr int HUMIDITY_BATHROOM = 7;			//2 bytes

	constexpr int PRESSURE = 8;								//2 bytes

	//fast package
	constexpr int PIRS = 0;										//1 byte
	constexpr int PIRS_UPDATED = 1;						//1 byte
	
	constexpr int LIGHT_BED = 2;							//2 bytes
	constexpr int LIGHT_DOOR = 3;							//2 bytes
	constexpr int LIGHT_KITCHEN = 4;					//2 bytes
}

//needed constants
constexpr uint8_t RADIO_ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes 
constexpr uint8_t REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; //TODO change to uint8_t if possible


constexpr uint8_t FASTDATA_SIZE = 5;
constexpr uint8_t SLOWDATA_SIZE = 9;
constexpr uint16_t SLOWDATA_COMPLETE = 0 | (1 << Idx::TEMPERATURE_BED) 
																				 | (1 << Idx::HUMIDITY_BED)
																				 | (1 << Idx::CO2)
																				 | (1 << Idx::PRESSURE);


#endif
