#ifndef MAINDATA_H
#define MAINDATA_H

#include <iostream>
#include <cstring> //memcopy
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <cstdlib>     /* abs */
#include <sys/stat.h> //mkdir and filesize
#include <ctime> //unix timestamp

#include "MainHeader.h"
#include "Search.h"
#include "Cache.h"


const static int MAXBLOCKSIZE = 512;

//keeps track of where data is located: file pointer, cacheSize, cache, 
//file path, and the oldest item in cache. During searches this is used to prevent
//the search function from leaving the transparent cache while unnecessary.
class Data : public Cache, public MainHeader
{
friend class Search;
public:
  Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize);

  /*gets the file pointer for setting shut down conditions*/
  FILE* getFileP();

  /* writes a package to file transparently caching it*/
  void append(uint8_t line[], uint32_t Tstamp);
  /* reads a single package at a given line*/
  void read(uint8_t line[], int lineNumber);
  /* reads all the lines including start to (excluding) start+length copies
     them to the array lines[]*/
  void readSeq(uint8_t line[], int start, int length);
  /* removes all lines between start and lengt, by inserting null data or
     in the case of a single line to be removed an extra timestamp package.
     Note in the file itself it is still clear that something has been removed
     this operation also does not save any space*/
  void remove(int lineNumber, int start, int length);

  //helper functions

  /*compares a pair of data packages and returns false if the first of them
    is a time package */
  bool notTSpackage(uint8_t lineA[], uint8_t lineB[]);
  /* write a full timestamp package to the data file and write the high part of the timestamp
   * to the header file together with the corresponding line number*/
  void putFullTS(const uint32_t Tstamp); //TODO debug header

private:
  /*keeps track if we have set the initial timestamp package*/
  bool initTimeStampNotSet;
  /*full unix time of previous package*/
  uint32_t prevTstamp;
  /*size of complete data packages*/
  uint8_t packageSize_;
  /*pointer to data file, created in the constructor during opening or creation of the datafile*/
  FILE* fileP_;
  /*path to which the constructor points*/
  std::string fileName_;
  /*return the unix time*/
  uint32_t unix_timestamp();
  /*TODO what should this do?*/
};


#endif // DATASTORE_H
