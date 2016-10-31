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
/*
Pir saving format, normal packages with sometimes a timestamp package in front

NORMAL PIR PACKAGE:
total length 4 bytes, time short contains the lower part of the 4 byte unix time
  ----------------------------------------------------------------------------
  - time low 16 bit | pir confirmed ones 8 bit | pir confirmed zeros 8 bit -
  ----------------------------------------------------------------------------

TIMESTAMP PIR PACKAGE:
total length 4 bytes, used to store the full unixtime just in front of a normal 
pir package that crosses a treshold for putting in the full time again 
  --------------------------------------
  - time low 16 bit | time high 16 bit -
  --------------------------------------
recognised by 2 time lows after eachother
--------------------------------------------
 
 
 ----HIGH LEVEL ACCESS (public) ----
    | storeData: storing data with a new time,*             -universal/all data
    | getting: data from specific time frames,              -universal/all data
    | removing specific time frames                         -universal/all data
 Locking mechanism to allow multi threaded/processed access (one lock per above action). implement write/read lock
 multiple readers or one write simultainiously
 
 *note: the full unix time is stored together with the raw data in a FIFO queue before the lock. The lock has a
        zero timeout. Thus time data is always accurate even if readout on another thread takes multiples of seconds
        //TODO is this needed? (might be, raspberry pi weak? implement last anyhow)
 
 ----LOW LEVEL ACCESS (private?) ----
 -Starting up:
    | fill cache: fills up the cache with stored data so we can access the stored
                  data quickly. The cache is implemented transparently.           -universal/all data

 -Storing a new time:
    | process:  adjust formatting                                                 -data specific
    | compress: check if it is really new data and needs to be saved              -data specific
    | package:  add all data together and add the timestamp part                  -data specific
  the above are all data specific. The below are not.
    | write: writes the package to both cache and file                            -universal/all data
    
 -getting data from specific time frames
    | searchFT: searches for the location of the two full timestamps closest to the requested
    |           unix times first                                                  -universal/all data
    | searchT:  searches onwards from the timestamps found in searchFT to find the lines of the exact times
    |           returns these lines                                               -universal/all data
    | getData:  given a binairy number for the wanted columns, returns a pointer to an array where it stores 
    |           pointers to the arrays containing the requested data              -universal/all data
*/           


//data specific functions and variables, inherits AllData
class PirData : public Data
{
  public:    
    PirData(std::string filePath, uint8_t* cache, uint8_t packageSize, int cacheLen)
    : Data(filePath, cache, packageSize, cacheLen);
    
    void process(uint8_t data[2]);

  private:
    StoreData dataStorage;
    
    struct timeval tp;//TODO cant this be in the function?  

    uint8_t prevData[2];    
    uint8_t Record[2];

    long int t_begin;

    bool TimeStampSet_first;
    bool TimeStampSet_second;    


    long int unix_timestamp();  

    uint8_t compress(uint8_t data);

    bool isTimeStampPackage(uint8_t susp_time[4],  uint8_t susp_data[4]);

    bool isNotSame(uint8_t data[2]);

    void convertNotation(uint8_t B[2]);
    void combine(uint8_t B[2]);
    void binData(uint8_t data[2]);

    void putData(uint8_t data[2]);  
    void putTimestamp(long int timestamp);        
};

#endif // DATASTORE_H
