#ifndef PIRDATA_H
#define PIRDATA_H
#include <iostream>
#include <cstring> //memcopy
#include <cstdint> //uint16_t

#include "MainData.h"

/*
pir (passive infrared receptor) data storage. Raw data is recieved from serial 
and processed. During processing the format is changed from on/off (1/0), 
sensor polled/not polled (1/0) => polled and one, polled and zero. The data to
a moniroring process and binned (PIR_DT second bintime) then stored on file.
*/

const int PIR_DT= 1; //time to bin data for
const int PACKAGESIZE = 2+2; //timestamp + data

//data specific functions and variables, inherits AllData
class PirData : public Data
{
  public:    
    PirData(const std::string filePath, uint8_t* cache, const int cacheLen);
    /*take the raw data from serial with the timestamp, rewrite it, send 
      it off for reacting if something changed and store it in a file*/
    void process(const uint8_t rawData[2], const uint32_t Tstamp);

  private:
    uint8_t polled_ones;    //confirmed detection by sensor (one= true)
    uint8_t polled_zeros;   //confirmed no detection (sensor has been polled!)

    uint8_t toStore_ones;    //value from previous run
    uint8_t toStore_zeros;   //value from previous run
    
    uint8_t prevRaw[2]; //keep the old data for testing if new data is diffrent
    uint32_t prevTstamp; //remember the timestamp of the previous run

    /*is the raw data diffrent then the previous data*/
    bool newData(const uint8_t rawData[2]);

    /*go from the notation one/zero sensor polled/no polled to the notation
      sensor polled High, sensor polled but Low*/
    void convertNotation(const uint8_t B[2]);

    /*bin data with old data*/
    void binData();
   
};

#endif // DATASTORE_H
