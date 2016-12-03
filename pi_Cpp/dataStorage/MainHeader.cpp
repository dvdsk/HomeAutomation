#include "MainHeader.h"
#include <sys/mman.h> //for mmap and mremap
#include <sys/stat.h> //for filesize and open
#include <fcntl.h> //open
#include <cstdint> //uint16_t
#include <sys/types.h> //lseek
#include <unistd.h> //lseek

#include <errno.h> //for human readable error
#include <string.h> //for human readable error

#include <assert.h>

const int BUFFERSIZE = 8*16; //allocate enough for just over a week

size_t MainHeader::getFilesize(const char* filename) {
    struct stat st;
    stat(filename, &st);
    return st.st_size;
}



int main(){
  MainHeader test("test.dat");
  test.append(12312,123214);
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
  
  //Execute mmap
  addr = mmap(NULL, mapSize, PROT_READ | PROT_WRITE, 
                           MAP_SHARED | MAP_POPULATE, fd, 0);
  assert(addr != MAP_FAILED);
 
  data = (uint32_t*)(addr);
 
  //Write the mmapped data to stdout (= FD #1)
  write(1, addr, filesize+BUFFERSIZE); 
}

MainHeader::~MainHeader(){
  //Cleanup
  int rc = munmap(addr, mapSize);
  assert(rc == 0);
  close(fd);
}

void MainHeader::append(uint32_t Tstamp, uint32_t byteNumber){
  uint32_t tostore = 43;
  data[1] = tostore;

////  uint8_t* p;
//  
//  if (pos == mappedspace-1){ 
//    std::cerr<<"expanding map";
//    //extend the memory map
//    mappedspace = mappedspace+BUFFERSIZE;
//    addr = mremap(addr, pos, mappedspace, MREMAP_MAYMOVE) ;  
//  }
//  //std::cerr<<"PUTTING SHIT IN MAP";

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
}

//void MainHeader::nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB){}



