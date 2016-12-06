#ifndef MAINHEADER_H
#define MAINHEADER_H

#include <cstdint> //uint16_t
#include <sys/stat.h> //mkdir and filesize
#include <iostream> //std::string

class MainHeader{
public:
  /* constructor, creates headerFile if it does not exist*/
  MainHeader(std::string fileName);
  void closeUp();
  /* appends a timestamp and the number of bytes from the beginning of
   * the data file to the header file*/
  void append(uint32_t Tstamp, uint32_t byteNumber);
  void showData(int atByte);  
  /* give the full timestamps closest to the given Tstamp*/
  //void nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB);

  int fd; //file discriptor 'points' to open file
  
  unsigned int pos; //position in header file in bytes
  uint32_t* data;
  void* addr; //adress where the memory map is placed
  //size_t mapSize; FIXME OLD
  size_t mapSize;

private:
  size_t getFilesize(const char* filename);
  
  void truncate(int fd, size_t& filesize);
  
};

#endif // MAINHEADER_H
