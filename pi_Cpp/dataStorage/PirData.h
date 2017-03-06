#ifndef PIRDATA_H
#define PIRDATA_H
#include <iostream>
#include <cstring> //memcopy
#include <cstdint> //uint16_t
#include <functional> //std::function

#include "MainData.h"
#include "../config.h"
/*
pir (passive infrared receptor) data storage. Raw data is recieved from serial 
and processed. During processing the format is changed from on/off (1/0), 
sensor polled/not polled (1/0) => polled and one, polled and zero. The data to
a moniroring process and binned (PIR_DT second bintime) then stored on file.
*/

//data specific functions and variables
class PirData : public Data
{
  public:    
    PirData(const std::string filePath, uint8_t* cache, const int cacheLen);
    /*take the raw data from serial with the timestamp, rewrite it, send 
      it off for reacting if something changed and store it in a file*/
    void process(const uint8_t rawData[2], const uint32_t Tstamp);

    /* fetches the full data from MainData. Reduces it to 2 arrays of PLOTRESOLUTION in length
     * Gives back the data in x and y arrays with that length. x (time axis) is cached */
    uint16_t fetchPirData(uint32_t startT, uint32_t stopT, double x[],
                          uint16_t y[]);

  private:
    uint8_t toStore_value;    //value gotten from sensors from previous runs
    uint8_t toStore_readSensores;   //value sensor has been polled previous runs
    
    uint8_t prevRaw[2]; //keep the old data for testing if new data is diffrent
    uint32_t prevTstamp; //remember the timestamp of the previous run

    /*is the raw data diffrent then the previous data*/
    bool newData(const uint8_t rawData[2]);
};

#endif // DATASTORE_H
