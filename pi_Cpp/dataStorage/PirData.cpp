#include "PirData.h"

//FIXME



PirData::PirData(const std::string filePath, uint8_t* cache, const int cacheLen)
: Data(filePath, cache, PACKAGESIZE, cacheLen){

  //init local variables
  toStore_ones = 0;
  toStore_zeros = 0;
  prevRaw[0] = 0;
  prevTstamp = 0;
}

void PirData::process(const uint8_t rawData[2], const uint32_t Tstamp){
  uint8_t line[2];
  
  if (newData(rawData) ){
    convertNotation(rawData);  
    //TODO send to responding function

    binData();
    if (Tstamp-prevTstamp >= PIR_DT){
      line[0] = toStore_zeros;
      line[1] = toStore_ones;
      
      Data::append(line, Tstamp);
      
      toStore_ones = 0;
      toStore_zeros = 0;
      prevTstamp = Tstamp;
    }
  }
}

/*given a block of binairy data containing pir packages, read data from
  one of the packages at blockIdx from the start of the block*/
float readSensorFromPackage(int orgIdx_B, int blockIdx_B, 
                            uint8_t block[MAXBLOCKSIZE], int extraParams[4]){
  std::cout<<"YESH WE GOT INTO TEST";
}

void PirData::fetchPirData(int sensor, uint32_t startT, uint32_t stopT, uint32_t x[], float y[]){
  int extraParams[4];//TODO we should encode pir scaling info in here too
  extraParams[0] = 4;
  Data::fetchData(startT, stopT, x, y, readSensorFromPackage, extraParams);
}

bool PirData::newData(const uint8_t raw[2]){
  if ((raw[0] == prevRaw[0]) & (raw[1] == prevRaw[1])){ return false;}
  else{
    std::memcpy(prevRaw, raw , 2);
    return true;
  }
}

void PirData::convertNotation(const uint8_t rawData[2]){
  uint8_t oneOrZero = rawData[0];
  uint8_t confirmed = rawData[1];

  polled_ones  =  oneOrZero & confirmed; //if one and noted as correct (one) give one
  polled_zeros = (oneOrZero ^ confirmed) & confirmed;
  
  /*explanation of confirmed zero algoritme
    we want only OnOff=F and confirmed=T to give T. the ^ (XOR) operator has 
    the following possible outcomes:
      OnOff:      F F T T
      confirmed:  F T F T
      outcome:    F T T F
    XOR gives us half of what we want. To filter out the confirmed=F case we
    do an AND operation. Now we have T where a zero is confirmed*/
}

void PirData::binData(){
  //expand registered sensors with previously registerd 
  //remembering only 1 second of old data (is forced in process)
  toStore_ones = toStore_ones | polled_ones;
  //expand list of confirmed zeros however force that they do not contradict
  //by forcing a zero in ones_toStore (AND NOT : force zero)
  toStore_zeros = (toStore_zeros | polled_zeros) & ~toStore_ones;
}


