#ifndef MAINHEADER_H
#define MAINHEADER_H

#ifdef DEBUG
#define db(x) std::cerr << x;
#else
#define db(x)
#endif

#include <cstdint> //uint16_t
#include <sys/stat.h> //mkdir and filesize
#include <iostream> //std::string
#include <sys/mman.h> //for mmap and mremap
#include <sys/stat.h> //for filesize and open
#include <fcntl.h> //open
#include <cstdint> //uint16_t
#include <sys/types.h> //lseek
#include <unistd.h> //lseek

#include <errno.h> //for human readable error
#include <string.h> //for human readable error

#include <fcntl.h> //fallocate
#include <unistd.h> //ftruncate
#include <sys/types.h> //ftruncate


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
  /* search for a Timestamp thats zero, from there on the file contains only
   * unused pre allocated data from the previous run. Delete all unused data */
  int fileSize(int fd, const char* filePath);
  
};

#endif // MAINHEADER_H
