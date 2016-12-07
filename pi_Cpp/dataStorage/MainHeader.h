#ifndef MAINHEADER_H
#define MAINHEADER_H

#include <cstdint> //uint16_t
#include <sys/stat.h> //mkdir and filesize
#include <iostream> //std::string

class MainHeader{
public:
  /* constructor, creates headerFile if it does not exist, if it does checks for
   * trailing zeros in the place of the time part. If these exist it will
   * truncate the file that position.*/
  MainHeader(std::string fileName);
  /* appends a timestamp and the number of bytes from the beginning of
   * the data file to the header file. If need be it will extend the mapping*/
  void append(uint32_t Tstamp, uint32_t byteNumber);
  /* test function*/
  void showData(int lineStart, int lineEnd);  
  
  /* give the line in the data file where the closest but smaller or egual then
   * the given timestamp is. In bytes from the beginning of the file where the
   * full timestamp starts.*/
  int findFullTS(uint32_t Tstamp);

  int fd; //file discriptor 'points' tou open file
  


private:
  unsigned int pos; //position in header file 'sizeof(uint32_t) bytes'
  uint32_t* data;
  void* addr; //adress where the memory map is placed
  //size_t mapSize; FIXME OLD
  size_t mapSize;

  size_t getFilesize(const char* filename);
  
  void truncate(int fd, size_t& filesize);
  
};

#endif // MAINHEADER_H
