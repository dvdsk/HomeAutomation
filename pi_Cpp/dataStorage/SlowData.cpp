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

bool SlowData::newData(const uint8_t raw[SLOWDATA_SIZE], uint16_t light_Mean[slowData::LIGHT_LEN]){
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

void SlowData::process(const uint8_t raw[SLOWDATA_SIZE], const uint32_t Tstamp){  
	uint8_t rawP[slowData::PACKAGESIZE-2]; //package without tstamp
	uint16_t light_Mean[3];
	for(int i = 0; i<slowData::LIGHT_LEN; i++){
  	light_Mean[i] = light_Sum[i]/light_N;
	}
	
	if(newData(raw, light_Mean)){
		//encode the light mean;
		std::memcpy(prevLight_Mean, light_Mean, slowData::LIGHT_LEN);
  	std::memcpy(prevRaw, raw, SLOWDATA_SIZE);				
		memcpy(rawP, raw, SLOWDATA_SIZE);
	
		encode(rawP, light_Mean[lght::BED], 		Enc_slow::LIGHT_BED, 		 Enc_slow::LEN_LIGHT);
		encode(rawP, light_Mean[lght::DOOR], 		Enc_slow::LIGHT_DOOR, 	 Enc_slow::LEN_LIGHT);
		encode(rawP, light_Mean[lght::KITCHEN], Enc_slow::LIGHT_KITCHEN, Enc_slow::LEN_LIGHT);

    Data::append(rawP, Tstamp); 
  }
}

//SPECIFIC DECODER FUNCT
float dTemp1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t temp_int = decode(block, blockIdx_B, Enc_slow::TEMP_BED, Enc_slow::LEN_TEMP);
	return (float(temp_int))/10 -10; }
float dTemp2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t temp_int = decode(block, blockIdx_B, Enc_slow::TEMP_BATHROOM, Enc_slow::LEN_TEMP);
	return (float(temp_int))/10 -10; }
float dTemp3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t temp_int = decode(block, blockIdx_B, Enc_slow::TEMP_DOOR, Enc_slow::LEN_TEMP);
	return (float(temp_int))/10 -10; }

float dHum1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t hum_int = decode(block, blockIdx_B, Enc_slow::HUM_BED, Enc_slow::LEN_HUM);
	return (float(hum_int))/10 -10; }//TODO REPLACE THESE CALC
float dHum2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t hum_int = decode(block, blockIdx_B, Enc_slow::HUM_BATHROOM, Enc_slow::LEN_HUM);
	return (float(hum_int))/10 -10; }//TODO REPLACE THESE CALC
float dHum3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t hum_int = decode(block, blockIdx_B, Enc_slow::HUM_DOOR, Enc_slow::LEN_HUM);
	return (float(hum_int))/10 -10; }//TODO REPLACE THESE CALC

float dLight1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t lightValue = decode(block, blockIdx_B, Enc_slow::LIGHT_BED, Enc_slow::LEN_LIGHT);
	return (float)lightValue;}
float dLight2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t lightValue = decode(block, blockIdx_B, Enc_slow::LIGHT_DOOR, Enc_slow::LEN_LIGHT);
	return (float)lightValue;}
float dLight3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t lightValue = decode(block, blockIdx_B, Enc_slow::LIGHT_KITCHEN, Enc_slow::LEN_LIGHT);
	return (float)lightValue;}

float dCo2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	uint16_t co2Val = decode(block, blockIdx_B, Enc_slow::CO2, Enc_slow::LEN_CO2);
	return (float)co2Val;}


//TODO possible optimisation using template and no longer a function pointer
int SlowData::fetchSlowData(uint32_t startT, uint32_t stopT, 
                            double x[], double y[], plotables sensor){
  int len;
  switch(sensor){
    case TEMP_BED:
      len = Data::fetchData(startT, stopT, x, y, dTemp1);   
      break;
    case TEMP_BATHROOM:
      len = Data::fetchData(startT, stopT, x, y, dTemp2);  
      break;
    case TEMP_DOORHIGH:
      len = Data::fetchData(startT, stopT, x, y, dTemp3);  
      break;    
    case HUMIDITY_BED:
      len = Data::fetchData(startT, stopT, x, y, dHum1);  
      break;    
    case HUMIDITY_BATHROOM:
      len = Data::fetchData(startT, stopT, x, y, dHum2);  
      break;    
    case HUMIDITY_DOORHIGH:
      len = Data::fetchData(startT, stopT, x, y, dHum3);  
      break;    
    case CO2PPM: 
      len = Data::fetchData(startT, stopT, x, y, dCo2);  
      break;
    case BRIGHTNESS_BED:
      len = Data::fetchData(startT, stopT, x, y, dLight1);  
      break; 
		default:
			std::cout<<"ERROR: INVALID CASE: "<<sensor<<"\n";
			break;   
  }  
  return len;
}

