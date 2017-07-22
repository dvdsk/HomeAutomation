#ifndef CACHE_H
#define CACHE_H
#include <iostream>
#include <cstring> //memcopy
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <cstdlib>     /* abs */
#include <sys/stat.h> //mkdir and filesize
#include <ctime> //unix timestamp

const int MAXPACKAGESIZE = 50;

class Cache
{
  public:
    /* set paramaters for the cache and check if these are correct*/
    Cache(const uint8_t packageSize, const int cacheSize );
    /* point the cache_ pointer to the cache and update cacheOldestT*/
    void InitCache(uint8_t* cache);
    /* copies an array to the cache, while overwriting old data*/
    void append(const uint8_t line[]);
    /* reads all the lines from start to start+length copies them to lines[]*/
    void readSeq(uint8_t line[], int start, int length);
    /* gets the low part of the first data in the cache*/
    uint16_t getFirstLowTime();
    /* search for a given timestamp in the cache return the cacheline*/
    int searchTimestamp(uint32_t Tstamp, int start, int stop);
    /* removes all lines between start and lengt, then updates the entire cache
     * filling it up again from file*/
    void remove(int lineNumber, int start, int length);

    /*length of cache in bytes (in uint8_t)*/
	  int cacheSize_;

  protected:
    /* pointer to cache, in the contructor we set this to an array, the cache has
     * multiple items getting newer as you get higher in the array*/
    uint8_t* cache_;
    /* indicates oldest (first added) element in the cache, we will overwrite this
     * first (in bytes)*/
    int cacheOldest_;
    /*size of complete data packages in bytes*/
    uint8_t packageSize_;
};


#endif // CACHE_H
