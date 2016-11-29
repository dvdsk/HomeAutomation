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
    /*set paramaters for the cache and check if these are correct*/
    Cache(const uint8_t packageSize, const int cacheSize );
    /*point the cache_ pointer to the cache and update cacheOldestT*/
    void InitCache(uint8_t* cache);
    
    /*copies an array to the cache, while overwriting old data, checks if the 
      overwritten data contained a full timestamp. If so updates cacheOldestT_
      [During startup the cache is filled and oldest timestamp initially set]*/
    void append(const uint8_t line[]);
    /*reads the package at a single line*/
    void read(uint8_t line[], int lineNumber);
    /*reads all the lines from start to start+length copies them to lines[]*/
    void readSeq(uint8_t line[], int start, int length);
    /*removes all lines between start and lengt, then updates the entire cache
      filling it up again from file*/
    void remove(int lineNumber, int start, int length);

    uint32_t cacheOldestT_;  //unix time of the oldest package in cache
    uint32_t lineOldest_;    //line number (in packages) of the oldest package in cache

    /*length of cache in bytes (in uint8_t)*/
	  int cacheSize_;

  private:
    /*pointer to cache, in the contructor we set this to an array, the cache has
      multiple items getting newer as you get higher in the array*/
    uint8_t* cache_;
    /*indicates oldest (first added) element in the cache, we will overwrite this
      first (in bytes)*/
    int cacheOldest_;
    /*size of complete data packages in bytes*/
    uint8_t packageSize_;
};


#endif // CACHE_H
