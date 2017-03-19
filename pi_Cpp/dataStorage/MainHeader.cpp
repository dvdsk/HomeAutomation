#include "MainHeader.h"

const unsigned int BUFFERSIZE = 4*2*sizeof(uint32_t); //allocate 16 lines of headers

size_t MainHeader::getFilesize(const char* filename) {
    struct stat st;
    stat(filename, &st);
    return st.st_size;
}

int MainHeader::fileSize(int fd, const char* filePath){
  //read in the last buffer;
  uint32_t data[BUFFERSIZE/sizeof(uint32_t)];
  int good_lines;
  int usefull;
  
  unsigned int filesize = getFilesize(filePath);
  std::cout<<"filesize: "<<filesize<<"\n";
  if(filesize < BUFFERSIZE){ return filesize;}
  filesize = filesize/2 *2; //make filesize even
  
  int startCheck = filesize-(1*(int)BUFFERSIZE);
  int stopCheck = filesize;
  
  lseek(fd, startCheck, SEEK_SET);
  int res = read(fd, &data, stopCheck-startCheck);
  std::cout<<"res: "<<res<<"\n";
  std::cout<<"start/stopcheck: "<<startCheck<<"/"<<stopCheck<<"\n";
  for(unsigned int i=0; i<(startCheck-stopCheck)/sizeof(uint32_t); i+=2) {
    std::cout<<"data["<<i<<"], byte: "<<i*4<<" = "<<data[i]<<"\n";
    if(data[i] == 0) {
      good_lines = i/2;
      usefull = good_lines *2*sizeof(uint32_t);
      
      std::cout<<"found data to be truncated\n";
      std::cout<<"i: "<<i<<" filesize: "<<filesize<<" usefull: "<<usefull
              <<" good_lines: "<<good_lines<<" res: "<<res
              <<" startCheck: "<<startCheck
              <<" stopCheck: "<<stopCheck
              <<" buffersize: "<<BUFFERSIZE<<"\n";

      filesize = startCheck+usefull;
      db("final file size: "<<filesize<<"\n")
      return filesize;
    }
  }
  return filesize;  
}

MainHeader::MainHeader(std::string fileName){
  const char* filePath;
  
  mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  fileName = ("data/"+fileName+".header");
  filePath = fileName.c_str();
 
  fd = open(filePath, O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IXUSR);
  
  size_t filesize = fileSize(fd, filePath);
  std::cerr<<"filesize: "<<filesize<<"\n";
  
  //exit(0);
  
  mapSize = filesize+BUFFERSIZE;
  db("mapSize: "<<mapSize<<"\n");
  pos = filesize/sizeof(uint32_t); //in elements
  db("startPos: "<<pos<<"\n");

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
 
  data = (uint32_t*)(addr);
}

void MainHeader::append(uint32_t Tstamp, uint32_t byteInDataFile){ 
  int oldSize;
  
  if ((pos)*sizeof(uint32_t) >= mapSize){
    db("expanding map\n");
    //extend the memory map
    oldSize = mapSize;
    mapSize = mapSize+BUFFERSIZE;
    addr = mremap(addr, oldSize, mapSize, MREMAP_MAYMOVE);
    data = (uint32_t*)(addr);
  
    //allocate space
    int result = fallocate(fd, 0, 0, mapSize);
    if (result == -1){std::cerr<<strerror(errno);}
  }
  
  db("pos: "<<pos<<"-"<<pos+1<<"\t\t byte: "<<(pos+1)*4<<"\n");
  db("mapSize: "<<mapSize<<"\n");
  data[pos+0] = Tstamp;
  data[pos+1] = byteInDataFile;
  
  //update the buffercounter and position in the file
  pos +=2;
  
}

//#ifdef DEBUG
void MainHeader::showData(int lineStart, int lineEnd){  
  std::cout<<"------------------------------\n";
  for(int i =lineStart*2; i<lineEnd*2; i+=2){
    std::cout<<"byte:  "<<i*4<<"\t";
    std::cout<<"Tstamp: "<<data[i+0]<<"\t";
    std::cout<<"byteInDataFile: "<<data[i+1]<<"\n";
  }
}
//#endif

void MainHeader::findFullTS(uint32_t Tstamp, int& A, int& B) {
  int low = 0;
  int high = (pos-2)/2; //as 'pos' points to free space
  uint32_t midData;
  int mid;
  int prevMid;
  //std::cout<<"\tsearching for Tstamp: "<<Tstamp<<"\n";
  //std::cout<<"\tdata[pos-2]: "<<data[pos-2]<<"\n";
  
  //check and handle edge cases TS not in data range
  if(Tstamp > data[pos-2]){
    A = data[pos-2+1];
    B = -1; //signals calling function that value is out of range
    std::cout<<"\twanted Tstamp larger then last full timestamp\n";
    return;
  }
  if(Tstamp < data[0]){
    A = 0;//Though the data wil not lie within this range
    B = data[1];//this will be detected in a later function
    std::cout<<"\twanted Tstamp smaller then first full timestamp\n";
    return;
  }
  
  while(low<=high){
    mid = low + ((high-low)/2);
    midData = data[mid*2];
    
    if(mid == prevMid){
      A = data[low*2+1];
      B = data[high*2+1];
      std::cout<<"\treturning: "<<A<<", "<<B<<"\n";
      return;
    }
    else if(midData == Tstamp){
      A = data[mid*2+1];
      B = data[high*2+1];
      std::cout<<"\treturning (found exact value): "<<A<<", "<<B<<"\n";
      return;
    }
    else if(midData > Tstamp){
      prevMid = mid;	
      high = mid;
    }
    else if(midData < Tstamp){
      prevMid = mid;	
      low = mid;
    }
  }
}

uint32_t MainHeader::lastFullTS(){
  if(pos < 2){ return 0;}
  else{ return data[pos-2];}
}

uint32_t MainHeader::fullTSJustBefore(unsigned int byte){
  //std::cerr<<"headerData: \n";
  //showData(0,pos);
  //std::cerr<<"pos: "<<pos<<"\n";
  for(int i = pos-2; i >= 0; i-=2){
    //std::cerr<<"i: "<<i<<"\n";
    //std::cerr<<"data[i+1]: "<<data[i+1]<<"\n";
    if(data[i+1] <= byte){
      //std::cerr<<i<<"\n";
      return data[i]; //return timestamp
    }
  }
  std::cerr<<"WARNING COULD NOT FIND TS BEFORE GIVEN BYTE: "<<+byte<<"\n";
  return -1;
}

void MainHeader::getNextFullTS(unsigned int byte, unsigned int& nextFullTSLoc, 
                               uint32_t& nextFullTS){
  for(unsigned int i = 0; i<= pos-2; i+=2){
    if(data[i+1] > byte){
      nextFullTS = data[i]; //return timestamp
      nextFullTSLoc = data[i+1];
      //std::cout<<"returning new timeHigh values: "<<nextFullTSLoc<<" \n";
      return;
    }
  }
  std::cout<<"setting to minus 1\n";
  nextFullTSLoc = -1;
  return;
}

//used for testing
#ifdef TESTO //test object file defined
int main(){
  int A;
  int B;

  MainHeader header("test");

  for(int i = 0; i<1; i++){
    header.append(1481034435+20+1*i,i);
  }
  header.showData(0,15);

  //header.findFullTS(1481034435+2*30, A, B);
  //std::cout<<"interval: "<<A<<", "<<B<<"\n";

  std::cout<<header.lastFullTS()<<"\n";
  return 0;
}
#endif
