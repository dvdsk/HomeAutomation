#ifndef MAINDATA_H
#define MAINDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <bitset>
#include <stdlib.h>     /* abs */

#include <sys/stat.h> //mkdir and filesize
#include <sys/time.h>

/*
Pir saving format, normal packages with sometimes a timestamp package in front

NORMAL PACKAGE:
total length 4 bytes, time short contains the lower part of the 4 byte unix time
  ----------------------------------------------------------------------------
  - time low 16 bit | N bits of usefull data                                 -
  ----------------------------------------------------------------------------

TIMESTAMP PACKAGE:
total length 4 bytes, used to store the full unixtime just in front of a normal 
pir package that crosses a treshold for putting in the full time again 
  --------------------------------------
  - time low 16 bit | N - 16 zeros     -
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

#define HALFDAYSEC 43200 //numb of sec in half a day
#define PIR_DT 1000 //number of milliseconds to bin the pir data to
#define KB 1000 //TODO replace with const shit (snap const shit eerst)
const int MAXPACKAGESIZE = 50;


class Cache
{
  public:
    /*set paramaters for the cache and check if these are correct*/
    Cache(uint8_t packageSize, int cacheSize );
    /*point the cache_ pointer to the cache and update cacheOldestT*/
    InitCache(uint8_t* cache);
    
    /*copies an array to the cache, while overwriting old data, checks if the 
      overwritten data contained a full timestamp. If so updates cacheOldestT_
      [During startup the cache is filled and oldest timestamp initially set]*/
    append(uint8_t line[]);
    /*reads the package at a single line*/
    read(uint8_t& line[], int lineNumber);
    /*reads all the lines from start to start+length copies them to lines[]*/
    readSeq(uint8_t& line[], int start, int length);
    /*removes all lines between start and lengt, then updates the entire cache
      filling it up again from file*/
    remove(int lineNumber, int start, int length);

    uint32_t cacheOldestT_;  //unix time of the oldest package in cache
    uint32_t lineOldest_;    //line number (in packages) of the oldest package in cache

  private:
    /*pointer to cache, in the contructor we set this to an array, the cache has
      multiple items getting newer as you get higher in the array*/
    uint8_t* cache_;
    /*length of cache in bytes (in uint8_t)*/
	  int cacheSize_;
    /*indicates oldest (first added) element in the cache, we will overwrite this
      first (in bytes)*/
    int cacheOldest_;
    /*size of complete data packages in bytes*/
    uint8_t packageSize_;
}

//keeps track of where data is located: file pointer, cacheSize, cache, 
//filepath, and the oldest item in cache. During searches this is used to prevent
//the search function from leaving the transparent cache while unnessesairy.
class Data : public Cache
{
  public:
    Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize) 
    : Cache(cache, packageSize, cacheLen)
    
    /*gets the file pointer for setting shut down conditions*/
    void getFileP();
    
    /*writes a package to file transparently caching it*/
    void append(uint8_t line[]);
    /*reads a single package at a given line*/
    void read(uint8_t& line[], int lineNumber);
    /*reads all the lines including start to (excluding) start+length copies
      them to the array lines[]*/
    void readSeq(uint8_t& line[], int start, int length);
    /*removes all lines between start and lengt, by inserting null data or
      in the case of a single line to be removed an extra timestamp package. 
      Note in the file itself it is still clear that something has been removed
      this operation also does not save any space*/    
    void remove(int lineNumber, int start, int length);
   
    /*searches for the line where a timestamp is located. First asks the cache
      if it contains the timestamp, depending on the awnser it starts searching
      in the cache or the file. Optionally a minimum point for the search can
      be given. [this is usefull for searching a second timestamp later timestamp]*/
    int searchTstamp(uint32_t Tstamp, int startLine = 0);
    
    //helper functions
    /*compares a pair of data packages and returns false if the first of them 
      is a timepackage */
    bool notTpackage(lineA, lineB, packageSize);
    
  protected:
    /*size of complete data packages*/
    uint8_t packageSize_;
    /*pointer to file, created in the constructor during opening or creation of
      the datafile*/
    FILE* fileP_;
    /*path to which the constructor points*/
    std::string fileName_;
};


#endif // DATASTORE_H
