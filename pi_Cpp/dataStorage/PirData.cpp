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
  std::cout << "in process\n";
  uint8_t line[2];
  
  if (newData(rawData) ){
    convertNotation(rawData);  
    //TODO send to responding function

    binData();
    if (Tstamp-prevTstamp > PIR_DT){
      line[0] = toStore_zeros;
      line[1] = toStore_ones;
      
      Data::append(line, Tstamp);
      
      toStore_ones = 0;
      toStore_zeros = 0;
      prevTstamp = Tstamp;
    }
  }
  std::cout << "leaving process\n";
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


