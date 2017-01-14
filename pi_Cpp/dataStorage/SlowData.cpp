#include "SlowData.h"

//package format:
//Temp: 9 bits        [storing -10.0 to 40.0 degrees, in values 0 to 500,
//                    values 501 means lower then -10.0 and 502 higher then 40.0]]
//Humidity: 10 bits   [storing 0.0 to 100.0 percent, in values 0 to 1000]
//Co2: 13 bits        [storing 0 to 6000ppm, in values 0 to 6000]

//[Tu, Td, Tb, Hu, Hd, Hb, Co2] = 70 bits. As we recieve it in bytes we ignore 
//the final 2 bytes.

SlowData::SlowData(const std::string filePath, uint8_t* cache, const int cacheLen)
: Data(filePath, cache, SLOWDATA_PACKAGESIZE, cacheLen){
  
  //init local variables
  memset(&prevRaw, 0, 9);
}

bool SlowData::newData(const uint8_t raw[9]){
  for(int i = 1; i<9; i++){
    if(raw[i] != prevRaw[i]){ return false;}
  }
  std::memcpy(prevRaw, raw, 9);
  return true;
}

void SlowData::process(const uint8_t raw[9], const uint32_t Tstamp){  
  if(newData(raw)){
    Data::append(raw, Tstamp); 
  }
}


//DECODE FUNCTIONS
float decodeTemperature1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  uint16_t temp_int;
  float temp;
  
  temp_int =  (uint16_t)block[blockIdx_B+0];
  temp_int = temp_int | ((uint16_t)(block[blockIdx_B+1] & 1)) << 8;
  temp = (float)temp_int;

  temp = temp/10 -10;
  
  return temp;
}

float decodeTemperature2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  uint16_t temp_int;
  float temp;
  
  temp_int = (uint16_t)((block[blockIdx_B+1] & 0b111111110)>>1);
  temp_int = temp_int | ((uint16_t)(block[blockIdx_B+2] & 0b00000011)) << 8;
  temp = (float)temp_int;

  temp = temp/10 -10;
  
  return temp;
}

float decodeTemperature3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  uint16_t temp_int;
  float temp;
  
  temp_int = (uint16_t)((block[blockIdx_B+1] & 0b111111110)>>1);
  temp_int = temp_int | ((uint16_t)(block[blockIdx_B+2] & 0b00000011)) << 8;
  temp = (float)temp_int;

  temp = temp/10 -10;
  
  return temp;
}

float decodeHumidity1(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float humidity;
  
  return humidity; 
}

float decodeHumidity2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float humidity;
  
  return humidity; 
}

float decodeHumidity3(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float humidity;
  
  return humidity; 
}

float decodeCO2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  float humidity;
  
  return humidity; 
}



int SlowData::fetchSlowData(uint32_t startT, uint32_t stopT, 
                            double x[], double y[], int sensor){
  int len;
  switch(sensor){
    case 1:
      len = Data::fetchData(startT, stopT, x, y, decodeTemperature1);   
      break;
    case 2:
      len = Data::fetchData(startT, stopT, x, y, decodeTemperature2);  
      break;
    case 3:
      len = Data::fetchData(startT, stopT, x, y, decodeTemperature3);  
      break;    
    case 4:
      len = Data::fetchData(startT, stopT, x, y, decodeHumidity1);  
      break;    
    case 5:
      len = Data::fetchData(startT, stopT, x, y, decodeHumidity2);  
      break;    
    case 6:
      len = Data::fetchData(startT, stopT, x, y, decodeHumidity3);  
      break;    
    case 7: 
      len = Data::fetchData(startT, stopT, x, y, decodeCO2);  
      break;
  }  
  return len;
}

