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
: Data(filePath, cache, SLOWDATA_PACKAGESIZE, cacheLen){
  
  //init local variables
  memset(&prevRaw, 0, 9);
}

bool SlowData::newData(const uint8_t raw[SLOWDATA_SIZE], uint16_t light_Mean[LIGHT_LEN]){
  for(int i = 1; i<9; i++){
    if(raw[i] == prevRaw[i]){ return false;}
  }
	for(int i = 0; i<LIGHT_LEN; i++){	
		if(light_Mean[lght::BED] == prevLight_Mean[lght::BED]){ return false;}
	}	

	std::memcpy(prevLight_Mean, light_Mean, LIGHT_LEN);
  std::memcpy(prevRaw, raw, SLOWDATA_SIZE);
  return true;
}

void SlowData::preProcess_light(uint8_t raw[FASTDATA_SIZE], const uint32_t Tstamp){  
	//add all the light values for averaging later
	light_Sum[lght::BED] += (uint32_t)decodeLight(Idx_fast::LIGHT_BED, raw); 
	//add other lights
	light_N ++;
}

void SlowData::process(const uint8_t raw[SLOWDATA_SIZE], const uint32_t Tstamp){  
	uint8_t rawP[SLOWDATA_PACKAGESIZE-2]; //package without tstamp
	uint16_t light_Mean[LIGHT_LEN];
	for(int i = 0; i<LIGHT_LEN; i++){
  	light_Mean[i] = light_Sum[i]/light_N;
	}
	
	if(newData(raw, light_Mean)){
		//encode the light mean;
		memcpy(rawP, raw, SLOWDATA_SIZE);
	
		for(int i = 0; i<LIGHT_LEN; i++){
			rawP[SLOWDATA_SIZE-LIGHT_LEN+i] 	= (uint8_t)(light_Mean[i] >> 8);	
			rawP[SLOWDATA_SIZE-LIGHT_LEN+i+1] = (uint8_t)(light_Mean[i]);	
		}		

    Data::append(rawP, Tstamp); 
  }
}

//DECODE FUNCT.
float decodeLight(int blockIdx_B, int bitOffSet, uint8_t block[MAXBLOCKSIZE]){
//blockIdx_B is the location in bytes from the start of block where
//the light data starts 
	uint16_t light_int;  
  float light = 2.0;

	light_int = ((uint16_t)(block[blockIdx_B] >> bitOffSet)) |
							((uint16_t)(block[blockIdx_B+1] << (8-bitOffSet));
	
	light = (float)light_int; //space for conversion formula
  return light; 
}

float decodeTemperature(int blockIdx_B, int bitOffSet, uint8_t block[MAXBLOCKSIZE]){
  uint16_t temp_int;
  float temp;
  
  temp_int =  (uint16_t)block[blockIdx_B+0];
  temp_int = temp_int | ((uint16_t)(block[blockIdx_B+1] & 1)) << 8;
  temp = (float)temp_int;

  temp = temp/10 -10;
  
  return temp;
}

float decodeHumidity(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float humidity = 2.0;

  return humidity; 
}

float decodeCO2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float co2 = 2.0;
  
  return co2; 
}

//SPECIFIC DECODER FUNCT
float dTemp1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
						 return decodeTemperature(blockIdx_B+Idx_slow::TEMP_BED, block); }
float dTemp2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
						 return decodeTemperature(blockIdx_B+Idx_slow::TEMP_BATHROOM, block); }
float dTemp3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
						 return decodeTemperature(blockIdx_B+Idx_slow::TEMP_DOOR, block); }

float dHum1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
					  return decodeHumidity(blockIdx_B+Idx_slow::HUM_BED, block); }
float dHum2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
					  return decodeHumidity(blockIdx_B+Idx_slow::HUM_BATHROOM, block); }
float dHum3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
					  return decodeHumidity(blockIdx_B+Idx_slow::HUM_DOOR, block); }

float dLight1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
						return decodeLight(blockIdx_B+Idx_slow::LIGHT_BED, block); }

//TODO possible optimisation using template and no longer a function pointer
int SlowData::fetchSlowData(uint32_t startT, uint32_t stopT, 
                            double x[], double y[], int sensor){
  int len;
  switch(sensor){
    case 1:
      len = Data::fetchData(startT, stopT, x, y, dTemp1);   
      break;
    case 2:
      len = Data::fetchData(startT, stopT, x, y, dTemp2);  
      break;
    case 3:
      len = Data::fetchData(startT, stopT, x, y, dTemp3);  
      break;    
    case 4:
      len = Data::fetchData(startT, stopT, x, y, dHum1);  
      break;    
    case 5:
      len = Data::fetchData(startT, stopT, x, y, dHum2);  
      break;    
    case 6:
      len = Data::fetchData(startT, stopT, x, y, dHum3);  
      break;    
    case 7: 
      len = Data::fetchData(startT, stopT, x, y, decodeCO2);  
      break;
    case 8:
      len = Data::fetchData(startT, stopT, x, y, dLight1);  
      break;    
  }  
  return len;
}

