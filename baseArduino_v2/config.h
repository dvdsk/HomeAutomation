#ifndef CONFIG
#define CONFIG

namespace config {
	constexpr int CALIBRATION_TIME = 2000; //milliseconds
	constexpr int READSPEED = 1; //millisec
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
	constexpr unsigned char FAST_UPDATE = 25;
	constexpr unsigned char SLOW_UPDATE = 26;
}

namespace radio {
	constexpr unsigned char NODE1_RQ_FAST_UPDATE = 1;
	constexpr unsigned char NODE2_RQ_FAST_UPDATE = 2;

	constexpr unsigned char NODE1_RQ_SLOW_UPDATE = 3;
	constexpr unsigned char NODE2_RQ_SLOW_UPDATE = 4;

	constexpr unsigned char NODE1_RQ_RESEND_SLOW = 5;
	constexpr unsigned char NODE2_RQ_RESEND_SLOW = 6;
}

//needed constants
constexpr uint8_t RADIO_ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes 
constexpr byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; //TODO change to uint8_t if possible

#endif

