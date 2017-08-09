#ifndef ENCODINGSCHEME
#define ENCODINGSCHEME

constexpr int roundUp(int a, int b){
	return (a+b-1)/b;
}

namespace EncSlowFile {
	//location where data starts in bits and lenght of data info		 
	constexpr int LEN_TEMP = 9;
	constexpr int LEN_HUM = 10;	
	constexpr int LEN_CO2 = 13;
	constexpr int LEN_PRESSURE = 16;
	constexpr int LEN_LIGHT = 10;

	//node bed
	constexpr int TEMP_BED = 0; 
	constexpr int HUM_BED = TEMP_BED+LEN_TEMP;		
	constexpr int CO2 = HUM_BED+LEN_HUM;	 
	constexpr int PRESSURE = CO2+LEN_CO2;

	//node kitchen
	constexpr int TEMP_BATHROOM = PRESSURE+LEN_PRESSURE; 
	constexpr int HUM_BATHROOM = TEMP_BATHROOM+LEN_TEMP;

	//node door
	constexpr int TEMP_DOOR = HUM_BATHROOM+LEN_HUM; 
	constexpr int HUM_DOOR = TEMP_DOOR+LEN_TEMP;		

	//lights
	constexpr int LIGHT_BED = HUM_DOOR+LEN_HUM;
	constexpr int LIGHT_KITCHEN = LIGHT_BED+LEN_LIGHT;
	constexpr int LIGHT_DOOR = LIGHT_KITCHEN+LEN_LIGHT;

	//startingPoints
	constexpr int START_BEDNODE = TEMP_BED;
	constexpr int START_KITCHEN = TEMP_BATHROOM;
	constexpr int START_DOOR = TEMP_DOOR;
	constexpr int START_LIGHTS = LIGHT_BED;

	//length
	constexpr int LEN_BEDNODE = roundUp(LEN_TEMP+LEN_HUM+LEN_CO2+LEN_PRESSURE+,8);
	constexpr int LEN_KITCHENNODE = roundUp(LEN_TEMP+LEN_HUM,8)
	constexpr int LEN_DOORNODE = roundUp(LEN_TEMP+LEN_HUM,8)
	constexpr int LEN_LIGHTS = roundUp(LEN_LIGHT*3, 8)

	constexpr int LEN_ENCODED = roundUp(START_LIGHTS+LEN_LIGHTS,8); //in bytes
}

namespace EncSlowArduino {
	//location where data starts in bits and lenght of data info		 
	constexpr int LEN_TEMP = 9;
	constexpr int LEN_HUM = 10;	
	constexpr int LEN_CO2 = 13;
	constexpr int LEN_PRESSURE = 16;

	//node bed
	constexpr int TEMP_BED = 0; 
	constexpr int HUM_BED = TEMP_BED+LEN_TEMP;		
	constexpr int CO2 = HUM_BED+LEN_HUM;	 
	constexpr int PRESSURE = CO2+LEN_CO2;
	constexpr int LEN_BEDNODE = roundUp(PRESSURE+LEN_PRESSURE,8); //in bytes 

	//node bathroom
	constexpr int TEMP_BATHROOM = 0; 
	constexpr int HUM_BATHROOM = TEMP_BATHROOM+LEN_TEMP;		

	//node kitchen
	constexpr int TEMP_DOOR = 0; 
	constexpr int HUM_DOOR = TEMP_DOOR+LEN_TEMP;		
}

namespace EncFastArduino {
	//location where data starts in bits and lenght of data info

	//need to stay at 0 and 1 for pirdata process to work
	constexpr int LEN_PIRS_BED = 2; //pirs + pirs_updated
	constexpr int LEN_PIRS_KICHEN = 5;
	constexpr int LEN_LIGHT = 10;

	//node bed
	constexpr int PIRS_BED = 1;
	constexpr int LIGHT_BED = PIRS_BED+LEN_PIRS_BED;
	constexpr int LEN_BED_NODE = roundUp(LIGHT_BED+LEN_LIGHT,8);

	//node kitchen
	constexpr int PIRS_KICHEN = 1;
	constexpr int LIGHT_KITCHEN = PIRS_KICHEN+LEN_PIRS_KICHEN;
	constexpr int LIGHT_DOOR = LIGHT_KITCHEN+LEN_LIGHT;
	constexpr int LEN_KITCHEN_NODE = roundUp(LIGHT_BED+LEN_LIGHT,8);
}

namespace EncFastFile {
	//location where data starts in bits and lenght of data info

	//need to stay at 0 and 1 for pirdata process to work
	constexpr int LEN_PIRS_BED = 2; //pirs + pirs_updated
	constexpr int LEN_PIRS_KICHEN = 5;

	//node bed
	constexpr int PIRS_BED = 0;
/*	constexpr int LIGHT_BED = PIRS_BED+LEN_PIRS_BED;*/
/*	constexpr int LEN_BED_NODE = roundUp(LIGHT_BED+LEN_LIGHT,8);*/

	//node kitchen
	constexpr int PIRS_KICHEN = PIRS_BED+LEN_PIRS_BED;
/*	constexpr int LIGHT_KITCHEN = PIRS_KICHEN+LEN_PIRS_KICHEN;*/
/*	constexpr int LIGHT_DOOR = LIGHT_KITCHEN+LEN_LIGHT;*/
/*	constexpr int LEN_KITCHEN_NODE = roundUp(LIGHT_BED+LEN_LIGHT,8);*/

	constexpr int LEN_ENCODED = roundUp(PIRS_KICHEN+LEN_PIRS_KICHEN,8); //in bytes
}

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




#endif

