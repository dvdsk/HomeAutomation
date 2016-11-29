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
  int searchTstamp(uint32_t Tstamp, int startLine = 0);

private:
  int findTimestamp_inFile(int startSearch, int StopSearch, uint32_t Tstamp);
  int findTimestamp_inCache(uint32_t Tstamp);
};

#endif // SEARCH_H
