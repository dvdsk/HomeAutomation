#ifndef CONFIG
#define CONFIG

#include <cstdint> //uint16_t
#include "encodingScheme.h"

//length in bytes
constexpr uint8_t FASTDATA_SIZE = 4;
constexpr uint8_t SLOWDATA_SIZE = 7;

constexpr uint16_t MAXPLOTRESOLUTION = 1000;

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF, MS_SLEEPING, MOVIEMODE};

namespace stateConf {
	constexpr int MAXMINIMALDURATION = 3600; //seconds: 1 hour
}

namespace headers {
	constexpr uint8_t SETUP_DONE = 200;
	constexpr uint8_t STARTUP_DONE = 201;
	constexpr uint8_t FAST_UPDATE = 255;
	constexpr uint8_t SLOW_UPDATE = 26;
}

namespace config {
	constexpr int HTTPSERVER_PORT = 8443;
	constexpr const char* HTTPSERVER_USER = "kleingeld";
	constexpr const char* HTTPSERVER_PASS = "nRhRudGLWs35rHukzxrz"; //using random strings as passw

	constexpr uint16_t ARDUINO_BAUDRATE = 9600;

	constexpr int ALERT_TEMP_ABOVE = 240; //in 0.1 Celcius, 24 deg
	constexpr int ALERT_TEMP_BELOW = 140; //in 0.1 Celcius, 14 deg
	
	constexpr int ALARM_TEMP_ABOVE = 350; //in 0.1 Celcius, 35 deg
	constexpr int ALARM_TEMP_BELOW = 50;  //in 0.1 Celcius, 5 deg
	
	constexpr int ALERT_HUMIDITY_ABOVE = 50; //in 0.1 Celcius, 24 deg
	constexpr int ALERT_HUMIDITY_BELOW = 30; //in 0.1 Celcius, 14 deg
	
	constexpr int ALARM_HUMIDITY_ABOVE = 90; //in 0.1 Celcius, 35 deg
	constexpr int ALARM_HUMIDITY_BELOW = 10;  //in 0.1 Celcius, 5 deg
	
	constexpr int ALERT_CO2PPM = 400;
	constexpr int ALARM_CO2PPM = 500;	
	
	constexpr int WCPIR_TIMEOUT = 60; //timout for bathroom lamp in seconds
	constexpr int KTCHN_TIMEOUT = 30; //timout for kitchen lamp in seconds
	
	constexpr int DT_HUMIDALARM_SHOWER = 600; //time allowed for humidity to
																						//drop in the bathroom
}

namespace lght {//lightvalues
	constexpr int BED = 0;
	constexpr int KITCHEN = 1;
	constexpr int DOOR = 2;
}

namespace mov {//movement sensors
	constexpr int DOOR = 0;
	constexpr int KITCHEN = 1;
	constexpr int BED_l = 2;
	constexpr int BED_r = 3;
	constexpr int RADIATOR = 4;
	constexpr int MIDDLEROOM = 5;
	constexpr int BATHROOM = 6;
}

namespace temp {//temp sensors
	constexpr const char* NAMES[]{"below bed\n", "in bathroom\n", "above door\n"};
	constexpr int BED = 0;
	constexpr int BATHROOM = 1;
	constexpr int DOOR = 2;
}

namespace hum {//humidity sensors
	constexpr const char* NAMES[]{"below bed\n", "in bathroom\n", "above door\n"};
	constexpr int BED = 0;
	constexpr int BATHROOM = 1;
	constexpr int DOOR = 2;
}

namespace lmp {//lamps
	constexpr int DOOR = 0;
	constexpr int KITCHEN = 1;
	constexpr int CEILING = 2;
	constexpr int BATHROOM = 3;
	constexpr int RADIATOR = 4;
	constexpr int BUREAU = 5;
}

namespace plnt {//plants
	constexpr const char* NAMES[]{"plantA", "plantB", "plantC"};
	constexpr int ALERT_HUMIDITY_BELOW[]{1, 2, 3};
	constexpr int NUMB_OF_PLANT_SENSORS = 3;
}


// THIS IS THE ENCODING USED BY DATASTORAGE TO STORE DATA IN MEMORY, IT 
// DIFFERS SUBTILY FROM THE ENCODING USED BY THE SENSORDATA
namespace Enc_slow {
	//location where data starts in bits and lenght of data info			 

	constexpr int LEN_LIGHT = 10;
	constexpr int LIGHT_BED = CO2+LEN_CO2;
	constexpr int LIGHT_DOOR = LIGHT_BED+LEN_LIGHT;
	constexpr int LIGHT_KITCHEN = LIGHT_DOOR+LEN_LIGHT;

	constexpr int LEN_ADD_ENCODED = LEN_LIGHT*3;
}

enum plotables{
  MOVEMENTSENSOR0,
  MOVEMENTSENSOR1,
  MOVEMENTSENSOR2,
  MOVEMENTSENSOR3,
  MOVEMENTSENSOR4,
  MOVEMENTSENSOR5,
  MOVEMENTSENSOR6,
  MOVEMENTSENSOR7,

  TEMP_BED,
  TEMP_BATHROOM,
  TEMP_DOORHIGH,

  HUMIDITY_BED,
  HUMIDITY_BATHROOM,
  HUMIDITY_DOORHIGH,

  CO2PPM,

  BRIGHTNESS_BED,
  BRIGHTNESS_BEYONDCURTAINS,
  BRIGHTNESS_KITCHEN,
  BRIGHTNESS_DOORHIGH
};

namespace mainState {
	constexpr int LEN_lightValues = 5;	
	constexpr int LEN_tempValues = 5;	
	constexpr int LEN_humidityValues = 5;		
	constexpr int LEN_soilHumidityValues = 5;		
	constexpr int LEN_movement = 5;	
}

namespace pirData {
	constexpr int PACKAGESIZE = Enc_fast::LEN_ENCODED+2;
	constexpr int PIR_DT= 1; //time to bin data for
}

namespace slowData {
	constexpr int PACKAGESIZE = Enc_slow::LEN_ENCODED+Enc_slow::LEN_ADD_ENCODED+2; 
	//slow data + light data + timestamp
}

#endif
