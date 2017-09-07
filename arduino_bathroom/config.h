#ifndef CONFIG
#define CONFIG

#include <Arduino.h> //needed for Serial.print

//note also change this in config of decoder class (USED FOR DECODING/ENCODING AIRPRESSURE)
constexpr uint32_t MINIMUM_MEASURABLE_PRESSURE = 93600; //Pa

namespace config {
	constexpr int CALIBRATION_TIME = 2000; //milliseconds
	constexpr int READSPEED = 1; //millisec
	constexpr int RESETSPEED = 500;
}

//pins
namespace pin {
	constexpr int TERM_DATA = A4; //PA2 (hard coded register banks)
	constexpr int TERM_CLOCK = A5; //PA0 (hard coded register banks)

	constexpr int LIGHT_BED = 6; //anolog in
	constexpr int PIR_BED_NORTH = 26; //PA4
	constexpr int PIR_BED_SOUTH = 28; //PA6

	constexpr int RADIO_CE = 7;
	constexpr int RADIO_CS = 8;
}

//node specific stuff in nodes ino file
namespace NODE_CENTRAL{
	constexpr uint8_t addr[] = "1Node"; //addr may only diff in first byte
}

constexpr uint8_t PIPE = 1;

namespace headers{
	constexpr uint8_t RQ_FAST = 0;
	constexpr uint8_t RQ_MEASURE_SLOW = 1;
	constexpr uint8_t RQ_READ_SLOW = 2;
	constexpr uint8_t RQ_INIT = 3;

	constexpr uint8_t INIT_DONE = 0b00000010;
	constexpr uint8_t SLOW_RDY = 0b00000001;
}

//needed constants
constexpr uint8_t RADIO_ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes 
constexpr byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; //TODO change to uint8_t if possible

#endif

