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


class MainHeader{
public:
  /* constructor, creates headerFile if it does not exist, if it does checks for
   * trailing zeros in the place of the time part. If these exist it will
   * truncate the file that position.*/
  MainHeader(std::string fileName);
  /* appends a timestamp and the number of bytes from the beginning of
   * the data file to the header file. If need be it will extend the mapping*/
  void append(uint32_t Tstamp, uint32_t byteNumber);
  
  //#ifdef DEBUG
  /* test function that shows whats in the file from lineStart to lineEnd*/
  void showData(int lineStart, int lineEnd);  
  //#endif
  
  /* give the line in the data file where the closest but smaller or equal then
   * the given timestamp is. In bytes from the beginning of the file where the
   * full timestamp starts. Also give the one after that.*/
  void findFullTS(uint32_t Tstamp, int& A, int& B);
  /* returns the time of the last set full timestamp*/
  uint32_t lastFullTS();
  /* given a location in the data file return the corrosponding full timestamp */
  uint32_t fullTSJustBefore(unsigned int byte);
  /* get the location and value of the next full timestamp */
  void getNextFullTS(unsigned int byte, unsigned int& nextFullTSLoc, 
                     uint32_t& nextFullTS);

	#ifdef DEBUG
	void showHeaderData(int lineStart, int lineEnd);
	int getCurrentLinepos(); //last line in units of LINESIZE
	void checkHeaderData();
	void showHeaderData();
	#endif

private:
	unsigned int pos; //next free spot in memory map in units 'sizeof(uint32_t) bytes'
  uint32_t* data; //adress used to place items into map
  void* addr; //adress where the memory map is placed
  //size_t mapSize; //size of the current memory mapping (used for data + allocated)
  int fd; //file discriptor 'points' tou open file
  size_t mapSize; //size of the current memory mapping (used for data + allocated)
  
  /* wrapper around stat to find the filesize in bytes*/
  size_t getFilesize(const char* filename);
  /* search for a Timestamp thats zero, from there on the file contains only
   * unused pre allocated data from the previous run. return only up until this
   * point. This is the abstract used filesize. In the contstructor this +  
   * BUFFERSIZE is allocated for*/
  int fileSize(int fd, const char* filePath);
  
};

#endif // MAINHEADER_H
