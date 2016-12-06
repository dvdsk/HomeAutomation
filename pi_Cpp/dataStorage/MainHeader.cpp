#include "MainHeader.h"
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

#include <assert.h>

const int BUFFERSIZE = 8*16; //allocate enough for just over a week

size_t MainHeader::getFilesize(const char* filename) {
    struct stat st;
    stat(filename, &st);
    return st.st_size;
}

size_t getFilesize(const char* filename) {
    struct stat st;
    stat(filename, &st);
    return st.st_size;
}


int main(){
  std::string fileName = "test.dat";
  const char* filePath;
  int mapSize;
  int pos = 0;
  void* addr;
  uint32_t* data;
  
  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  filePath = ("data/"+fileName).c_str();
  
  int fd = open(filePath, O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IXUSR);

  assert(fd != -1);
  size_t filesize = getFilesize(filePath);
  std::cout<<"fileSize: "<<filesize<<"\n";
  
  mapSize = filesize+BUFFERSIZE;
  pos = filesize;

  //Make the file big enough (only allocated chrashes with new file)
  lseek (fd, mapSize, SEEK_SET);
  write (fd, "", 1);
  
  //allocate space
  //int result = fallocate(fd, 0, 0, mapSize);
  //if (result == -1){std::cerr<<strerror(errno);}
  
  
  //Execute mmap
  addr = mmap(NULL, mapSize, PROT_READ | PROT_WRITE, 
                             MAP_SHARED | MAP_POPULATE, fd, 0);
  assert(addr != MAP_FAILED);
 
  data = (uint32_t*)(addr);
 
  //Write the mmapped data to stdout (= FD #1)
  write(1, addr, filesize);
  std::cout<<"\n";   

  //int result = ftruncate(fd, pos);
  //std::cerr<<"pos: "<<pos;
  //if (result == -1){std::cerr<<strerror(errno);}
  //std::cerr<<"BOOOB\n";
  //std::cerr<<"fd: "<<fd<<"\n";
  
  data[pos+0] = 1;
  data[pos+1] = 2;
  data[pos+2] = 3;
  data[pos+3] = 4;
  data[pos+4] = 5;
  data[pos+5] = 6;
  pos = 6;
  
  //flush to file
  int result = ftruncate(fd, pos*4);
  std::cerr<<"pos: "<<pos<<"\n";
  if (result == -1){std::cerr<<strerror(errno);}

  std::cerr<<"syncing\n";
  result = msync(addr, mapSize, MS_SYNC); //asyncronus 
  if (result == -1){std::cerr<<strerror(errno);}

  //Cleanup
  std::cerr<<"unmapping\n";
  int rc = munmap(addr, mapSize);
  assert(rc == 0);
  std::cerr<<"fd_close: "<<fd<<"\n";

  std::cerr<<"closing file\n";
  result = close(fd);
  if(result == -1){std::cerr<<strerror(errno);}

  return 0;
}


MainHeader::MainHeader(std::string fileName){
  const char* filePath;
  
  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  filePath = ("data/"+fileName).c_str();
  
  int fd = open(filePath, O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IXUSR);

  assert(fd != -1);
  size_t filesize = getFilesize(filePath);
  std::cout<<"fileSize: "<<filesize<<"\n";
  
  mapSize = filesize+BUFFERSIZE;
  pos = filesize;

  //Make the file big enough (only allocated chrashes with new file)
  lseek (fd, mapSize, SEEK_SET);
  write (fd, "", 1);
  
  //allocate space
  //int result = fallocate(fd, 0, 0, mapSize);
  //if (result == -1){std::cerr<<strerror(errno);}
  
  
  //Execute mmap
  addr = mmap(NULL, mapSize, PROT_READ | PROT_WRITE, 
                             MAP_SHARED | MAP_POPULATE, fd, 0);
  assert(addr != MAP_FAILED);
 
  data = (uint32_t*)(addr);
 
  //Write the mmapped data to stdout (= FD #1)
  write(1, addr, filesize);
  std::cout<<"\n"; 
  
}

void MainHeader::closeUp(){
  //flush to file
  int result = ftruncate(fd, pos);
  std::cerr<<"pos: "<<pos<<"\n";
  if (result == -1){std::cerr<<strerror(errno);}

  std::cerr<<"syncing\n";
  result = msync(addr, mapSize, MS_SYNC); //asyncronus 
  if (result == -1){std::cerr<<strerror(errno);}

  //Cleanup
  std::cerr<<"unmapping\n";
  int rc = munmap(addr, mapSize);
  assert(rc == 0);
  std::cerr<<"fd_close: "<<fd<<"\n";

  std::cerr<<"closing file\n";
  result = close(fd);
  if(result == -1){std::cerr<<strerror(errno);}
}



void MainHeader::append(uint32_t Tstamp, uint32_t byteInDataFile){ 
  std::cerr<<"fd: "<<fd<<"\n";
  
  if (pos*8 == mapSize-1){
    std::cerr<<"expanding map";
    //extend the memory map
    mapSize = mapSize+BUFFERSIZE;
    addr = mremap(addr, pos, mapSize, MREMAP_MAYMOVE);
  }
  //std::cerr<<"PUTTING SHIT IN MAP";
  
  std::cout<<"pos: "<<pos<<"\n";
  data[pos+0] = Tstamp;
  data[pos+1] = byteInDataFile;
  //msync(addr, pos, MS_SYNC);
  
  if (msync(addr, pos, MS_SYNC) == -1)
  {
    std::cerr<<("Could not sync the file to disk");
  }
  
  //update the buffercounter and position in the file
  pos +=2;
  
}

void MainHeader::read(int atByte, uint32_t& Tstamp, uint32_t& byteInDataFile){  
  int seek = 0;

  Tstamp = data[seek+0];
  byteInDataFile = data[seek+1];
  
  std::cout<<"Tstamp: "<<Tstamp<<"\n";
  std::cout<<"byteInDataFile: "<<byteInDataFile<<"\n";
}

//void MainHeader::nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB){}



