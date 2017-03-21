#include "SlowData.h"

//package format:
//Temp: 9 bits        [storing -10.0 to 40.0 degrees, in values 0 to 500,
//                    values 501 means lower then -10.0 and 502 higher then 40.0]]
//Humidity: 10 bits   [storing 0.0 to 100.0 percent, in values 0 to 1000]
//Co2: 13 bits        [storing 0 to 6000ppm, in values 0 to 6000]
//Light 10 bits				[storing unedited output of arduino ADC]

//[Tu, Td, Tb, Hu, Hd, Hb, Co2] = 70 bits. As we recieve it in bytes we ignore 
//the final 2 bytes.

SlowData::SlowData(const std::string filePath, uint8_t* cache, const int cacheLen)
: Data(filePath, cache, slowData::PACKAGESIZE, cacheLen){
  
  //init local variables
  memset(&prevRaw, 0, 9);
}

bool SlowData::newData(const uint8_t raw[Enc_slow::LEN_ENCODED], uint16_t light_Mean[3]){
	for(int i = 1; i<9; i++){
    if(raw[i] != prevRaw[i]){ return true;}
  }
	if(light_Mean[lght::BED] 		 != prevLight_Mean[lght::BED]){ return true;}
	if(light_Mean[lght::KITCHEN] != prevLight_Mean[lght::KITCHEN]){ return true;}
	if(light_Mean[lght::DOOR] 	 != prevLight_Mean[lght::DOOR]){ return true;}
	
	return false;
}

void SlowData::preProcess_light(int lightValues[], const uint32_t Tstamp){  
	//add all the light values for averaging later
	light_Sum[lght::BED] 			+= lightValues[lght::BED]; 
	light_Sum[lght::KITCHEN] 	+= lightValues[lght::KITCHEN]; 
	light_Sum[lght::DOOR] 		+= lightValues[lght::DOOR]; 
	light_N ++;
}

void SlowData::process(const uint8_t raw[Enc_slow::LEN_ENCODED], const uint32_t Tstamp){  
	uint8_t raw_extended[Enc_slow::LEN_ENCODED+Enc_slow::LEN_ADD_ENCODED]; //package without tstamp
	uint16_t light_Mean[3];

	/*calculate the mean of all light data since the last slowdata package*/
	for(int i = 0; i<3; i++){
  	light_Mean[i] = light_Sum[i]/light_N;
	}
	
	if(newData(raw, light_Mean)){
		//encode the light mean;
		std::memcpy(prevLight_Mean, light_Mean, 3);
  	std::memcpy(prevRaw, raw, Enc_slow::LEN_ENCODED);				
		
		memcpy(raw_extended, raw, Enc_slow::LEN_ENCODED);
		memset(raw_extended+Enc_slow::LEN_ENCODED, 0, Enc_slow::LEN_ADD_ENCODED);		
		
		encode(raw_extended, light_Mean[lght::BED], Enc_slow::LIGHT_BED, Enc_slow::LEN_LIGHT);
		encode(raw_extended, light_Mean[lght::DOOR], Enc_slow::LIGHT_DOOR, Enc_slow::LEN_LIGHT);
		encode(raw_extended, light_Mean[lght::KITCHEN], Enc_slow::LIGHT_KITCHEN, Enc_slow::LEN_LIGHT);

    Data::append(raw_extended, Tstamp); 
  }
}

//SPECIFIC DECODER FUNCT
/* +2 converts arduino packages to packages with a 2 byte timestamp in front*/
uint16_t dTemp1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::TEMP_BED, Enc_slow::LEN_TEMP);}
uint16_t dTemp2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::TEMP_BATHROOM, Enc_slow::LEN_TEMP);}
uint16_t dTemp3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::TEMP_DOOR, Enc_slow::LEN_TEMP);}

double tempToFloat(uint16_t integer_var){return integer_var/10. -10; }

uint16_t dHum1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::HUM_BED, Enc_slow::LEN_HUM); }
uint16_t dHum2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::HUM_BATHROOM, Enc_slow::LEN_HUM); }
uint16_t dHum3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::HUM_DOOR, Enc_slow::LEN_HUM); }

double humToFloat(uint16_t integer_var){return integer_var/10.; }

uint16_t dLight1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::LIGHT_BED, Enc_slow::LEN_LIGHT); }
uint16_t dLight2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::LIGHT_DOOR, Enc_slow::LEN_LIGHT); }
uint16_t dLight3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::LIGHT_KITCHEN, Enc_slow::LEN_LIGHT); }

uint16_t dCo2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, Enc_slow::CO2, Enc_slow::LEN_CO2); }

double toFloat(uint16_t integer_var){return (double)integer_var; }


//TODO possible optimisation using template and no longer a function pointer
int SlowData::fetchSlowData(uint32_t startT, uint32_t stopT, 
                            double x[], double y[], plotables sensor){
  int len;
  switch(sensor){
    case TEMP_BED:
      len = Data::fetchData(startT, stopT, x, y, dTemp1, tempToFloat);   
      break;
    case TEMP_BATHROOM:
      len = Data::fetchData(startT, stopT, x, y, dTemp2, tempToFloat);  
      break;
    case TEMP_DOORHIGH:
      len = Data::fetchData(startT, stopT, x, y, dTemp3, tempToFloat);  
      break;    
    case HUMIDITY_BED:
      len = Data::fetchData(startT, stopT, x, y, dHum1, humToFloat);  
      break;    
    case HUMIDITY_BATHROOM:
      len = Data::fetchData(startT, stopT, x, y, dHum2, humToFloat);  
      break;    
    case HUMIDITY_DOORHIGH:
      len = Data::fetchData(startT, stopT, x, y, dHum3, humToFloat);  
      break;    
    case CO2PPM: 
      len = Data::fetchData(startT, stopT, x, y, dCo2, toFloat);  
      break;
    case BRIGHTNESS_BED:
      len = Data::fetchData(startT, stopT, x, y, dLight1, toFloat);  
      break; 
		default:
			std::cout<<"ERROR: INVALID CASE: "<<sensor<<"\n";
			break;   
  }  
  return len;
}

