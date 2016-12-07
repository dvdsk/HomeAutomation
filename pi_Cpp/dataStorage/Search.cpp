#include "Search.h"

#include <iostream>

int Search::searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2) {
  uint32_t Tstamp;

  Tstamp = Tstamp1;
  //check if the wanted timestamp could be in the cache
  if (Tstamp > Data::cacheOldestT_){
    findTimestamp_inCache(Tstamp);
  }
  else{
    findTimestamp_inFile(Tstamp);
  }
  return closestLine;
}

int Search::findTimestamp_inFile(uint32_t Tstamp){
  uint8_t block[MAXBLOCKSIZE];
  int startSearch;
  int stopSearch;
  int Idx;

  // check the full timestamp file to get the location of the full timestamp still smaller then
  // but closest toTstamp and the next full timestamp (that is too large thus). No need to catch the case
  // where the Full timestamp afther Tstamp does not exist as such a Tstamp would result into seaching in cache.
  startSearch = MainHeader::findFullTS(Tstamp, startSearch, stopSearch);


  // find maximum block size, either the max that fits N packages, or the space between stop and start
  int blockSize = std::min(MAXBLOCKSIZE-(MAXBLOCKSIZE%packageSize_), StopSearch-startSearch);

  Idx = -1;
  fseek(fileP_, startSearch, SEEK_SET);
  do {
    fread(block, blockSize, 1, fileP_); //read everything into the buffer

    Idx = searchBlock(block, Tstamp, blockSize);
    // check through the block for a timestamp
  } while(Idx == -1);

  if(Idx == -1){ return stopSearch; }
  else{ return Idx+startSearch;}
}

int Search::searchBlock(uint8_t block[], uint16_t Tstamplow, int blockSize) {
  uint16_t timelow;

  for(int i = 0; i<blockSize; i+=packageSize_){
    timelow = (uint16_t)*(block[i+1]) << 8  |
              (uint16_t)*(block[i]);
    if(timelow > Tstamplow){//then
      return i-packageSize_;
    }
  }
  return -1;
}

int Search::findTimestamp_inCache(uint32_t Tstamp){
}
