#ifndef ENCODINGSCHEME
#define ENCODINGSCHEME

namespace Enc_slow {
	//location where data starts in bits and lenght of data info
	constexpr int updated = 0;				 

	constexpr int TEMP_BED = 1; 
	constexpr int TEMP_BATHROOM = 3; 
	constexpr int TEMP_DOOR = 5; 
	constexpr int LEN_TEMP = 4;

	constexpr int HUM_BED = 2;		 
	constexpr int HUM_BATHROOM = 2;		
	constexpr int HUM_DOOR = 2;		
	constexpr int LEN_HUM = 2;	

	constexpr int CO2 = 3;						 
	constexpr int LEN_CO2 = 3;
	
	constexpr int LEN_ENCODED = 10; //in bytes
}

namespace Enc_fast {
	//location where data starts in bits and lenght of data info

	//need to stay at 0 and 1 for pirdata process to work
	constexpr int PIRS = 0;
	constexpr int PIRS_UPDATED = 16;
	constexpr int LEN_PIRS = 32;

	constexpr int LIGHT_BED = 2;
	constexpr int LIGHT_DOOR = 2+10;
	constexpr int LIGHT_KITCHEN = 2+10+10;
	constexpr int LEN_LIGHT = 10;

	constexpr int LEN_ENCODED = 10; //in bytes
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

