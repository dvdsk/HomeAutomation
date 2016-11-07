#ifndef PIRDATA_H
#define PIRDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <bitset>

#include <sys/stat.h> //mkdir and filesize
#include <sys/time.h>

#include "MainData.h"


//data specific functions and variables, inherits AllData
class PirData : public Data
{
  public:    
    PirData(std::string filePath, uint8_t* cache, uint8_t packageSize, int cacheLen);
    /*take the raw data from serial with the timestamp, rewrite it, send 
      it off for reacting if something changed and store it in a file*/
    void process(uint8_t rawData[2], uint32_t Tstamp);

  private:
    uint8_t confirmed_ones;    //confirmed detection by sensor (one= true)
    uint8_t confirmed_zeros;   //confirmed no detection (sensor has been polled!)

    uint8_t toStore_ones;    //value from previous run
    uint8_t toStore_zeros;   //value from previous run
    
    uint8_t prevRaw[2]; //keep the old data for testing if new data is diffrent
    uint32_t prevTstamp; //remember the timestamp of the previous run

    /*is the raw data diffrent then the previous data*/
    bool newData(uint8_t rawData[2]);

    /*go from the notation one/zero sensor polled/no polled to the notation
      sensor polled High, sensor polled but Low*/
    void convertNotation(uint8_t B[2]);

    /*bin data with old data*/
    void binData();
   
};

#endif // DATASTORE_H
