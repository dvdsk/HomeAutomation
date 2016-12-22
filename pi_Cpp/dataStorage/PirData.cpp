#include "PirData.h"

//FIXME



PirData::PirData(const std::string filePath, uint8_t* cache, const int cacheLen)
: Data(filePath, cache, PACKAGESIZE, cacheLen){

  //init local variables
  toStore_value = 0;
  toStore_readSensores = 0;
  prevRaw[0] = 0;
  prevTstamp = 0;
}


void PirData::process(const uint8_t rawData[2], const uint32_t Tstamp){
  uint8_t line[2];
  
  std::cout<<"processing\n";
  if (newData(rawData) ){
    //bin data
    toStore_value = toStore_value | rawData[0];
    toStore_readSensores = toStore_readSensores | rawData[1];

    if (Tstamp-prevTstamp >= PIR_DT){
      line[0] = toStore_value;
      line[1] = toStore_readSensores;
      
      std::cout<<"storing: "<<+toStore_value<<", "<<+toStore_readSensores<<"\n";
      
      Data::append(line, Tstamp);
      
      toStore_value = 0;
      toStore_readSensores = 0;
      prevTstamp = Tstamp;
    }
  }
}

/*given a block of binairy data containing pir packages, read data from
  one of the packages at blockIdx from the start of the block*/
float readSensorFromPackage(int orgIdx_B, int blockIdx_B, 
                            uint8_t block[MAXBLOCKSIZE], int extraParams[4]){
  float f;
  uint8_t values;
  uint8_t readSensores;
  
  //encode into 1 float 
  values = block[blockIdx_B+2];
  readSensores = block[blockIdx_B+3];

  uint32_t temp = 0;//TODO check if we can make this a uint16
  temp = (((uint32_t)values << 8) | (uint32_t)readSensores);
  f = *((float*)&temp);
  return f;
}

uint16_t PirData::fetchPirData(int sensor, uint32_t startT, uint32_t stopT, 
                               uint32_t x[], float y[]){
  int extraParams[4];
  return Data::fetchData(startT, stopT, x, y, readSensorFromPackage, extraParams);
}

bool PirData::newData(const uint8_t raw[2]){
  if ((raw[0] == prevRaw[0]) & (raw[1] == prevRaw[1])){ return false;}
  else{
    std::memcpy(prevRaw, raw , 2);
    return true;
  }
}

