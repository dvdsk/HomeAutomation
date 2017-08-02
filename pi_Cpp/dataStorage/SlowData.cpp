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
: Data(filePath, cache, EncSlowFile::LEN_ENCODED, cacheLen){
  
  //init local variables
  memset(&prevRaw, 0, 9);
}

bool SlowData::newData(const uint8_t raw[EncSlowArduino::LEN_ENCODED], uint16_t light_Mean[3]){
	for(int i = 1; i<9; i++){
    if(raw[i] != prevRaw[i]){ return true;}
  }
	if(light_Mean[lght::BED] 		 != prevLight_Mean[lght::BED]){ return true;}
	if(light_Mean[lght::KITCHEN] != prevLight_Mean[lght::KITCHEN]){ return true;}
	if(light_Mean[lght::DOOR] 	 != prevLight_Mean[lght::DOOR]){ return true;}
	
	return false;
}

void SlowData::preProcess_light(std::atomic<int> lightValues[], uint8_t lightId, const uint32_t Tstamp){  
	//add all the light values for averaging later
	light_Sum[lightId] += lightValues[lightId]; 
	light_N[lightId]++;
}

void SlowData::process(const uint8_t raw[EncSlowArduino::LEN_ENCODED], const uint32_t Tstamp){  
	uint8_t raw_extended[EncSlowFile::LEN_ENCODED]; //package without tstamp
	uint16_t light_Mean[3];

	/*calculate the mean of all light data since the last slowdata package*/
	for(int i = 0; i<3; i++){
		if(light_N[i] != 0){			
			light_Mean[i] = light_Sum[i]/light_N[i];
			light_Sum[i] = 0;
		}		
		else light_Mean[i] = prevLight_Mean[i];
		light_N[i] = 0;
	}

	if(newData(raw, light_Mean)){
		//encode the light mean;
		std::memcpy(prevLight_Mean, light_Mean, 3);
  	std::memcpy(prevRaw, raw, EncSlowArduino::LEN_ENCODED);				
		
		memcpy(raw_extended, raw, EncSlowArduino::LEN_ENCODED);
		memset(raw_extended+EncSlowArduino::LEN_ENCODED, 0, 
		       EncSlowFile::LEN_ENCODED-EncSlowArduino::LEN_ENCODED);		
		
		encode(raw_extended, light_Mean[lght::BED], EncSlowFile::LIGHT_BED, EncSlowFile::LEN_LIGHT);
		encode(raw_extended, light_Mean[lght::DOOR], EncSlowFile::LIGHT_DOOR, EncSlowFile::LEN_LIGHT);
		encode(raw_extended, light_Mean[lght::KITCHEN], EncSlowFile::LIGHT_KITCHEN, EncSlowFile::LEN_LIGHT);

    Data::append(raw_extended, Tstamp); 
  }
}

//SPECIFIC DECODER FUNCT
/* +2 converts arduino packages to packages with a 2 byte timestamp in front*/
uint16_t dTemp1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::TEMP_BED, EncSlowFile::LEN_TEMP);}
uint16_t dTemp2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::TEMP_BATHROOM, EncSlowFile::LEN_TEMP);}
uint16_t dTemp3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::TEMP_DOOR, EncSlowFile::LEN_TEMP);}

float tempToFloat(uint16_t integer_var){return integer_var/10. -10; }

uint16_t dHum1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::HUM_BED, EncSlowFile::LEN_HUM); }
uint16_t dHum2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::HUM_BATHROOM, EncSlowFile::LEN_HUM); }
uint16_t dHum3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::HUM_DOOR, EncSlowFile::LEN_HUM); }

float humToFloat(uint16_t integer_var){return integer_var/10.; }

uint16_t dLight1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::LIGHT_BED, EncSlowFile::LEN_LIGHT); }
uint16_t dLight2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::LIGHT_DOOR, EncSlowFile::LEN_LIGHT); }
uint16_t dLight3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::LIGHT_KITCHEN, EncSlowFile::LEN_LIGHT); }

uint16_t dCo2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::CO2, EncSlowFile::LEN_CO2); }

float toFloat(uint16_t integer_var){return (float)integer_var; }

uint16_t dPressure(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
	return decode(block, blockIdx_B+2, EncSlowFile::PRESSURE, EncSlowFile::LEN_PRESSURE); }

float pressureToFloat(uint16_t integer_var){return integer_var/5+MINIMUM_MEASURABLE_PRESSURE; }

//TODO possible optimisation using template and no longer a function pointer
int SlowData::fetchSlowData(uint32_t startT, uint32_t stopT, 
                            uint32_t x[], float y[], plotables sensor){
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
    case PRESSURE:
      len = Data::fetchData(startT, stopT, x, y, dPressure, pressureToFloat);  
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

void SlowData::exportAllSlowData(uint32_t startT, uint32_t stopT){
	unsigned int startByte = 0;
	unsigned int stopByte = 0;
	uint32_t x[MAX_FETCHED_ELEMENTS];
	float y[MAX_FETCHED_ELEMENTS];
	int len;

	std::fstream fs;
	fs.open ("SlowData.txt", std::fstream::out | std::fstream::trunc);

	//fetches all data in a loop;
	do{
		len = Data::fetchAllData(startT, stopT, startByte, stopByte, x, y, dLight1, toFloat);
		//std::cout<<startByte<<", "<<stopByte<<", "<<x[len-1]<<", "<<y[len-1]<<"\n";		
		for(int i=0; i<len; i++){
			fs<<x[i]<<" "<<y[i]<<"\n";
		}
	}while(startByte<stopByte);

	fs.close();
}
