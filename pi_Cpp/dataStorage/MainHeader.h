#ifndef MAINHEADER_H
#define MAINHEADER_H

#include <cstdint> //uint16_t
#include <sys/stat.h> //mkdir and filesize
#include <iostream> //std::string

class MainHeader{
public:
  /* constructor, creates headerFile if it does not exist*/
  MainHeader(std::string fileName);
  /* gets the headerFile pointer for setting shut down conditions*/
//  uint32_t* getHFileP(); //TODO implement header
  /* appends a timestamp and the number of bytes from the beginning of the data file to the header file*/
  void append(uint32_t Tstamp, uint32_t byteNumber);
  /* give the full timestamps closest to the given Tstamp*/
  void nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB);
private:
  /* pointer to the header file, created in the constructor during opening or creation of
   * the header file*/
  FILE* hFileP_;
  void* addr; //adress where the memory map is placed
  int* map;
  int pos;
  int mappedspace;
  
};

#endif // MAINHEADER_H
