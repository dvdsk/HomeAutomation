#ifndef CONFIG
#define CONFIG

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF, MS_SLEEPING, MOVIEMODE};

namespace stat {
	constexpr int MAXMINIMALDURATION = 3600; //seconds: 1 hour
}

namespace config {
	constexpr int HTTPSERVER_PORT = 8443;
	constexpr const char* HTTPSERVER_USER = "kleingeld";
	constexpr const char* HTTPSERVER_PASS = "nRhRudGLWs35rHukzxrz"; //using random strings as passw
	
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
	constexpr int DOOR = 0;
	constexpr int KITCHEN = 1;
	constexpr int BED = 2;
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

namespace hum {//movement sensors
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

//dont forget to update in the arduino config file
namespace Idx {
	constexpr int updated = 0;	
	constexpr int co2 = 3;
	constexpr int temperature_bed = 1;
	constexpr int humidity_bed = 2;

	constexpr int pirs = 0;
	constexpr int pirs_updated = 1;
	constexpr int light_bed = 2;
}

constexpr uint8_t FASTDATA_SIZE = 4;
constexpr uint8_t SLOWDATA_SIZE = 9;

#endif
