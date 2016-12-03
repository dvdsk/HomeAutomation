#include "MainHeader.h"
#include <sys/mman.h> //for mmap and mremap
#include <sys/stat.h> //for filesize and open
#include <fcntl.h> //open
#include <cstdint> //uint16_t
#include <sys/types.h> //lseek
#include <unistd.h> //lseek

#include <errno.h> //for human readable error
#include <string.h> //for human readable error

#include <assert.h>//FIXME

const int BUFFERSIZE = 8*16; //allocate enough for just over a week

size_t getFilesize(const char* filename) {
    struct stat st;
    stat(filename, &st);
    return st.st_size;
}



int main(){
  int* test;
  
  std::string fileName = "test.dat";
  const char* filePath;
  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  filePath = ("data/"+fileName).c_str();
  
  int fd = open(filePath, O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IXUSR);
  assert(fd != -1);
  size_t filesize = getFilesize(filePath);
  std::cout<<filesize;
  
  //Execute mmap
  void* mmappedData = mmap(NULL, filesize+BUFFERSIZE, PROT_READ | PROT_WRITE, 
                           MAP_SHARED | MAP_POPULATE, fd, 0);
  assert(mmappedData != MAP_FAILED);
 
  test = (int*)(mmappedData);
  int tostore = +"a";
  test[0] = tostore;
 
  //Write the mmapped data to stdout (= FD #1)
  write(1, mmappedData, filesize+BUFFERSIZE);
  
  
  
  //Cleanup
  int rc = munmap(mmappedData, filesize+BUFFERSIZE);
  assert(rc == 0);
  close(fd);
}


//MainHeader::MainHeader(std::string fileName){
//  int fd; //file discriptor
//  struct stat fileStats;
//  
//  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
//  fileName = "data/"+fileName;
//  //ask kernel to open the file
//  fd = open(fileName.c_str(), O_RDWR | O_CREAT );

//  //set current position in the map/file to the end of the file
//  stat(fileName.c_str(), &fileStats);
//  pos = fileStats.st_size;  
//  std::cout<<"pos: "<<pos<<"\n";
//  mappedspace = pos+BUFFERSIZE;
//   
//  //allocate space for the buffer
//  lseek(fd, mappedspace-1, SEEK_SET);
//  write(fd, "", 1);
// 
//  addr =mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);  
//  if (addr == MAP_FAILED)
//  {
//    std::cerr << "ERROR: could not map the file." << std::endl;
//  }
//  map = (int*)(addr);

//}


//void MainHeader::append(uint32_t Tstamp, uint32_t byteNumber){
////  uint8_t* p;
//  
//  if (pos == mappedspace-1){ 
//    std::cerr<<"expanding map";
//    //extend the memory map
//    mappedspace = mappedspace+BUFFERSIZE;
//    addr = mremap(addr, pos, mappedspace, MREMAP_MAYMOVE) ;  
//  }
//  //std::cerr<<"PUTTING SHIT IN MAP";

//  //addr[0] = 2;

//  //write the timestamp to the map
////  p = (uint8_t*)&Tstamp;
////  addr[pos+0] = p[0];
////  addr[pos+1] = p[1];
////  addr[pos+2] = p[2];
////  addr[pos+3] = p[3];

//  //write the corrosponding byteNumber
////  p = (uint8_t*)&byteNumber;
////  addr[pos+4] = p[0];
////  addr[pos+5] = p[1];
////  addr[pos+6] = p[2];
////  addr[pos+7] = p[3];
//  
//  //update the buffercounter and position in the file
//  pos +=8;
//}

//void MainHeader::nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB){}



