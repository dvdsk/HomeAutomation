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

const unsigned int BUFFERSIZE = 16*2*sizeof(uint32_t); //allocate 16 lines of headers

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
  MainHeader header("test.dat");

  header.append(1481034435,0);
  header.append(1481034435,0);
  header.append(1481034435,0);
  header.append(1481034435,0);

//  header.append(0,0);
//  header.append(0,0);
//  header.append(0,0);
//  header.append(0,0);

  header.showData(0);
  header.showData(10);

  header.closeUp();
  
  return 0;
}

void MainHeader::truncate(int fd, size_t& filesize){
//  search for a Timestamp thats zero, that must be unused allocated data from
//  the previous run.
  //read in the last buffer;
  uint32_t data[BUFFERSIZE/sizeof(uint32_t)];
  int good_lines;
  int usefull;
  int result; 
  
  result = lseek(fd, -1*(int)BUFFERSIZE, SEEK_END);
  if (result == -1){std::cerr<<strerror(errno)<<"\n";}    
  
  int res = read(fd, &data, BUFFERSIZE);
  std::cerr<<"read: "<<res<<" bytes\n";
  
  //find out there the file needs to be truncated
  for(unsigned int i=0; i<res/sizeof(uint32_t); i+=2) {
    //std::cout<<+dataB[i]<<"\n";
    if(data[i] == 0) {
      good_lines = (i)/2;
      usefull = good_lines *2*sizeof(uint32_t);
      
      std::cerr<<"found data to be truncated\n";
      std::cerr<<"i: "<<i<<" filesize: "<<filesize<<" linesFound: "<<usefull/(2*sizeof(uint32_t))
               <<" buffersize: "<<BUFFERSIZE<<"\n";
      
      filesize = filesize-BUFFERSIZE+ usefull-1;
      break;
    }
  }
  
  result = ftruncate(fd, filesize);
  if (result == -1){std::cerr<<strerror(errno)<<"\n";}    
}

MainHeader::MainHeader(std::string fileName){
  const char* filePath;
  
  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  filePath = ("data/"+fileName).c_str();
 
  fd = open(filePath, O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IXUSR);
  
  assert(fd != -1);
  size_t filesize = getFilesize(filePath);
  if (filesize > BUFFERSIZE){ truncate(fd, filesize);}
  std::cout<<"fileSize: "<<filesize<<"\n";
  
  //exit(0);
  
  mapSize = filesize+BUFFERSIZE;
  std::cerr<<"mapSize: "<<mapSize<<"\n";
  pos = filesize/sizeof(uint32_t); //in elements
  std::cerr<<"startPosSize: "<<pos<<"\n";

  //Make the file big enough (only allocated chrashes with new file)
  lseek (fd, mapSize, SEEK_SET);
  int result = write (fd, "", 1);
  if (result == -1){std::cerr<<strerror(errno);}
  
  //allocate space
  result = fallocate(fd, 0, 0, mapSize);
  if (result == -1){std::cerr<<strerror(errno);}
  
  
  //Execute mmap
  addr = mmap(NULL, mapSize, PROT_READ | PROT_WRITE, 
                             MAP_SHARED | MAP_POPULATE, fd, 0);
  assert(addr != MAP_FAILED);
 
  data = (uint32_t*)(addr);
}

void MainHeader::closeUp(){
//  int result;

//  //flush to file
//  result = ftruncate(fd, pos*sizeof(uint32_t));
//  std::cerr<<"truncating file to length: "<<pos*sizeof(uint32_t)<<"\n";
//  if (result == -1){std::cerr<<strerror(errno)<<"\n";}

/* SEEMS OPTIONAL
  //std::cerr<<"syncing\n";
  result = msync(addr, mapSize, MS_SYNC); //asyncronus 
  if (result == -1){std::cerr<<strerror(errno)<<"\n";}

  //Cleanup
  std::cerr<<"unmapping\n";
  int rc = munmap(addr, mapSize);
  assert(rc == 0);

  std::cerr<<"closing file\n";
  result = close(fd);
  if(result == -1){std::cerr<<strerror(errno);}
*/
}



void MainHeader::append(uint32_t Tstamp, uint32_t byteInDataFile){ 
  
  if (pos == mapSize-1){
    std::cerr<<"expanding map";
    //extend the memory map
    mapSize = mapSize+BUFFERSIZE;
    addr = mremap(addr, pos, mapSize, MREMAP_MAYMOVE);
  }
  //std::cerr<<"PUTTING SHIT IN MAP";
  
  std::cout<<"pos: "<<pos<<"-"<<pos+1<<"\n";
  data[pos+0] = Tstamp;
  data[pos+1] = byteInDataFile;
  
  //update the buffercounter and position in the file
  pos +=2;
  
}

void MainHeader::showData(int atByte){  
  
  std::cout<<"Tstamp: "<<data[atByte+0]<<"\n";
  std::cout<<"byteInDataFile: "<<data[atByte+1]<<"\n";
}

//void MainHeader::nearest2FullTS(uint32_t Tstamp, uint32_t FTSA, uint32_t FTSB){}


