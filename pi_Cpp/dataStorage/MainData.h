#ifndef MAINDATA_H
#define MAINDATA_H

#ifdef DEBUG
#define db(x) std::cerr << x;
#else
#define db(x)
#endif

#include <iostream>
#include <cstring> //memcopy
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <cstdlib>     /* abs */
#include <sys/stat.h> //mkdir and filesize
#include <ctime> //unix timestamp

#include "MainHeader.h"
#include "Cache.h"

const static unsigned int MAXBLOCKSIZE = 512000; //512kb

//keeps track of where data is located: file pointer, cacheSize, cache, 
//file path, and the oldest item in cache. During searches this is used to prevent
//the search function from leaving the transparent cache while unnecessary.
class Data : public Cache, public MainHeader
{
public:
  Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize);

  /*gets the file pointer for setting shut down conditions*/
  FILE* getFileP();

  /* writes a package to file transparently caching it*/
  void append(const uint8_t line[], const uint32_t Tstamp);

  void showLines(int start_P, int end_P);

  /* takes a start and end time + an array to store the time in and one to store
   * floats in. Binning of data happens if there are 2* or more data points then
   * the plotresolution. The data is plotted from the start time to and including
   * the end time*/
  int fetchData(uint32_t startT, uint32_t stopT, double x[], double y[],
	              uint16_t (*func)(int blockIdx_B, uint8_t[MAXBLOCKSIZE]), 
	              double (*func2)(uint16_t integer_var));

  /* variand of fetchData that does not return floats but uses uint16 type and
   * performs bitwise or operations instead of meaning */
  int fetchBinData(uint32_t startT, uint32_t stopT, double x[], uint16_t y[],
                   uint16_t (*func)(int blockIdx_B, uint8_t[MAXBLOCKSIZE]));

  /* removes all lines between start and lengt, by inserting null data or
     in the case of a single line to be removed an extra timestamp package.
     Note in the file itself it is still clear that something has been removed
     this operation also does not save any space*/
  void remove(int lineNumber, int start, int length);

  /* searches for the line where a timestamp is located. First asks the cache
   if it contains the timestamp, depending on the awnser it starts searching
   in the cache or the file. Optionally a minimum point for the search can
   be given. [this is useful for searching a second timestamp later timestamp]*/
  void searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2, unsigned int& loc1, unsigned int& loc2);

  //HELPER FUNCT
  /* compares a pair of data packages and returns false if the first of them
   * is a time package */
  bool notTSpackage(uint8_t lineA[], uint8_t lineB[]);
  /* write a full timestamp package to the data file and write the high part of the timestamp
   * to the header file together with the corresponding line number*/
  void putFullTS(const uint32_t Tstamp); //TODO debug header
  /* generate indexes we will not take into accout for reading*/
  class iterator;


private:
  /*full unix time of previous package*/
  uint32_t prevFTstamp;
  /*size of complete data packages*/
  uint8_t packageSize_;
  /*pointer to data file, created in the constructor during opening or creation of the datafile*/
  FILE* fileP_;
  /*path to which the constructor points*/
  std::string fileName_;

  //GENERAL HELP FUNCT
  /*return the unix time*/
  uint32_t unix_timestamp();

  //SEARCH FUNCT
  /* search for the timestamp in the cache, this is done in a 'dumb' way due to 
   * caching in the processor  we start at the beginning of the cache and iterate
   * through it checking for the requested time*/
  int findTimestamp_inCache(uint32_t Tstamp, unsigned int startSearch, 
                            unsigned int stopSearch, unsigned int fileSize);
  /* given a start and stop searchpoint these functions will search for
   * respectively a timestamp from the bottem up and top down. Reading in chunks to
   * make the process more efficient. Returns the best value in the range*/
  int findTimestamp_inFile_lowerBound(uint16_t TS_low, unsigned int startSearch,
                                      unsigned int stopSearch);
  int findTimestamp_inFile_upperBound(uint32_t TS, unsigned int startSearch,
                                      unsigned int stopSearch);
  
  /*possible full timestamp part of the previous package*/
  uint8_t prevTimePart[4];
  /*high High part of the last full timestamp*/
  uint32_t timeHigh;
  
  /*initialise the time reconstruct aloritme*/
  void initGetTime(int startByte);
  /*get the full unix time from the last full timestamp high part en the low
   *part of the current package.*/
  uint32_t getTime(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]);
  
  /*calculate the mean of an array of uint32_t*/
  double meanT(uint32_t* array, int len);
  /*calculate the mean, (in this case the bitwise or product)*/
  uint16_t meanB(uint16_t* array, int len);
};

class Data::iterator {
public:
  iterator(unsigned int startByte, unsigned int stopByte, unsigned int packageSize_);
  bool useValue(unsigned int i);

  int binSize_P;
private:
  float spacing;
  float counter;
};

#endif // DATASTORE_H
