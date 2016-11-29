#include "Search.h"

#include <iostream>

int Search::searchTstamp(uint32_t Tstamp, int startLine){//TODO
  struct stat filestatus;
  int fileSize; 
  uint32_t closest; //keep track of the closest timestamp found
  int closestLine; //line of the closest timestamp 
  int cacheStart; //point in file where the cache starts having a copy, (bytes)

  std::cout<<Data::fileP_;   //FIXME ONLY HERE FOR TESTING

  //if the wanted timestamp is not in the file
  if (Tstamp < Data::cacheOldestT_){
    //get file info
    stat(fileName_.c_str(), &filestatus);//sys call for file info
    fileSize = filestatus.st_size;
    
    //till here searching makes sense (as things can not be in the cache)
    cacheStart = fileSize-Cache::cacheSize_
    //TODO file flush
    findTimestamp_inFile(0 , cacheStart, Tstamp)
  }
  else{
    findTimestamp_inCache
  }
  return closestLine;
}

int Search::findTimestamp_inFile(int startSearch, int StopSearch, uint32_t Tstamp){
  uint8_t TSA[2];
  uint8_t TSB[2];
  uint8_t readBuf[MAXBLOCKSIZE];
  int i = 0;

  //check the full timestamp file to get the full timestamp shortest before Tstamp


  startSearch = startSearch+(startSearch-StopSearch)/2;

  //find maximum suitable block size
  int blockSize = std::min(MAXBLOCKSIZE-(MAXBLOCKSIZE%packageSize_), StopSearch-startSearch);

  fseek(fileP_, startSearch, SEEK_SET);
  fread(readBuf, blockSize, 1, fileP_); //read everything into the buffer

  //check through the block for a timestamp
  do {
    TSA = readBuf+i;
    TSB = TSA;
    i+=packageSize_;
  } while(notTSpackage(TSA, TSB) && i<blockSize);
  

}

int Search::findTimestamp_inCache(uint32_t Tstamp){
}
