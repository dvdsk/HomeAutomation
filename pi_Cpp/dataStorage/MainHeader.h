#ifndef MAINHEADER_H
#define MAINHEADER_H

#include <cstdint> //uint16_t
#include <sys/stat.h> //mkdir and filesize
#include <iostream> //std::string

class MainHeader{
public:
  /* constructor, creates headerFile if it does not exist*/
  MainHeader(std::string fileName);
  ~MainHeader();
  /* appends a timestamp and the number of bytes from the beginning of
   * the data file to the header file*/
  void append(uint32_t Tstamp, uint32_t byteNumber);
  /* give the full timestamps closest to the given Tstamp*/
  //void nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB);
private:
  size_t mapSize;
  int fd; //file discriptor 'points' to open file
  void* addr; //adress where the memory map is placed
  uint32_t* data;
  
  size_t getFilesize(const char* filename);
};

#endif // MAINHEADER_H
