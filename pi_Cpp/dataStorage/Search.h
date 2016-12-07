#ifndef SEARCH_H
#define SEARCH_H

#include "MainData.h"
#include <algorithm>

class Search{
friend class Data;
public:
  /* searches for the line where a timestamp is located. First asks the cache
     if it contains the timestamp, depending on the awnser it starts searching
     in the cache or the file. Optionally a minimum point for the search can
     be given. [this is useful for searching a second timestamp later timestamp]*/
  int searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2);

private:
  /* search for the timestamp in the cache, this is done in a 'dumb' way due to caching in the processor.
   * we start at the beginning of the cache and iterate through it checking for the requested time*/
  int findTimestamp_inCache(uint32_t Tstamp);
  /* The header file is asked for the full timestamps surrounding Tstamp, we then load the data in between into
   * memory block for block and iterate through it searching for the requested time*/
  int findTimestamp_inFile(uint32_t Tstamp);

  /* search a block read into memory for the value closest to Tstamp*/
  int searchBlock(uint8_t block[], uint16_t T_lower);
};

#endif // SEARCH_H
