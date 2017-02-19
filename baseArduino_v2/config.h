#ifndef CONFIG
#define CONFIG

//the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways
typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

namespace fDat {
	constexpr int PIRS = 0;
	constexpr int LIGHT_BED = 1;
}

namespace sdat {
	constexpr int CO2 = 0;
}

namespace config {
	constexpr int CALIBRATION_TIME = 1; //milliseconds
	constexpr int READSPEED = 1; //millisec
	constexpr int RESETSPEED = 500;
}

//pins
namespace pin {
	constexpr int TERM_DATA = 24; //PB2 (hard coded register banks)
	constexpr int TERM_CLOCK = 22; //PB0 (hard coded register banks)

	constexpr int LIGHT_BED = 0; //anolog in

	constexpr int RADIO_CE = 7;
	constexpr int RADIO_CS = 9;
}

namespace headers {
	constexpr unsigned char SETUP_DONE = 200;
	constexpr unsigned char FAST_UPDATE = 255;
	constexpr unsigned char SLOW_UPDATE = 26;
}

namespace radioRQ {
	constexpr unsigned char NODE1_FAST_UPDATE = 1;
	constexpr unsigned char NODE2_FAST_UPDATE = 2;

	constexpr unsigned char NODE1_SLOW_UPDATE = 3;
	constexpr unsigned char NODE2_SLOW_UPDATE = 4;

	constexpr unsigned char NODE1_RESEND_SLOW = 5;
	constexpr unsigned char NODE2_RESEND_SLOW = 6;
}

namespace Idx {
	constexpr int updated = 0;	
	constexpr int co2 = 3;
	constexpr int temperature_bed = 1;
	constexpr int humidity_bed = 2;

	constexpr int pirs = 0;
	constexpr int pirs_updated = 1;
	constexpr int light_bed = 2;
}

//needed constants
constexpr uint8_t RADIO_ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes 
constexpr byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; //TODO change to uint8_t if possible

constexpr uint8_t FASTDATA_SIZE = 4;
constexpr uint8_t SLOWDATA_SIZE = 9;
constexpr uint16_t SLOWDATA_COMPLETE = 0 | (1 << Idx::temperature_bed) 
																				 | (1 << Idx::humidity_bed)
																				 | (1 << Idx::co2);


#endif

