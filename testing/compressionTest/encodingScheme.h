#ifndef ENCODINGSCHEME
#define ENCODINGSCHEME

constexpr int roundUp(int a, int b){
	return (a+b-1)/b;
}

namespace Enc_slow {
	//location where data starts in bits and lenght of data info		 
	constexpr int LEN_TEMP = 9;
	constexpr int TEMP_BED = 0; 
	constexpr int TEMP_BATHROOM = TEMP_BED+LEN_TEMP; 
	constexpr int TEMP_DOOR = TEMP_BATHROOM+LEN_TEMP; 

	constexpr int LEN_HUM = 10;	
	constexpr int HUM_BED = TEMP_DOOR+LEN_TEMP;		 
	constexpr int HUM_BATHROOM = HUM_BED+LEN_HUM;		
	constexpr int HUM_DOOR = HUM_BATHROOM+LEN_HUM;		

//	constexpr int LEN_CO2 = 13;
//	constexpr int CO2 = HUM_DOOR+LEN_HUM;						 

//	constexpr int LEN_PRESSURE = 16;
//	constexpr int PRESSURE = CO2+LEN_CO2;

	constexpr int LEN_PRESSURE = 16;
	constexpr int PRESSURE = HUM_DOOR+LEN_HUM;						 

	constexpr int LEN_CO2 = 13;
	constexpr int CO2 = PRESSURE+LEN_PRESSURE;
	
	constexpr int LEN_ENCODED = roundUp(PRESSURE+LEN_PRESSURE,8); //in bytes
}

namespace Enc_fast {
	//location where data starts in bits and lenght of data info

	//need to stay at 0 and 1 for pirdata process to work
	constexpr int LEN_PIRS = 32; //pirs + pirs_updated
	constexpr int PIRS = 0;
	constexpr int PIRS_UPDATED = 16;

	constexpr int LEN_LIGHT = 10;
	constexpr int LIGHT_BED = LEN_PIRS;
	constexpr int LIGHT_DOOR = LIGHT_BED+LEN_LIGHT;
	constexpr int LIGHT_KITCHEN = LIGHT_DOOR+LEN_LIGHT;

	constexpr int LEN_ENCODED = roundUp(LIGHT_KITCHEN+LEN_LIGHT,8); //in bytes
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

