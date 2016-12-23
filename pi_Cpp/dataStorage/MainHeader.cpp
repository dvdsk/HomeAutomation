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
  int result; 
  
  size_t filesize = getFilesize(filePath);
  db("filesize: "<<filesize<<"\n");
  if(filesize < BUFFERSIZE){ return filesize;}
  result = lseek(fd, -1*(int)BUFFERSIZE, SEEK_END);
  if (result == -1){std::cerr<<strerror(errno)<<"\n";}    
  
  int res = read(fd, &data, BUFFERSIZE);
  db("read: "<<res<<" bytes\n");
  
  //find out there the file needs to be truncated
  for(unsigned int i=0; i<res/sizeof(uint32_t); i+=2) {
    if(data[i] == 0) {
      good_lines = (i)/2;
      usefull = good_lines *2*sizeof(uint32_t);
      
      db("found data to be truncated\n");
      db("i: "<<i<<" filesize: "<<filesize<<" linesFound: "<<usefull/(2*sizeof(uint32_t))
               <<" buffersize: "<<BUFFERSIZE<<"\n");

      filesize = filesize-BUFFERSIZE+usefull-1;
      break;
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
  
  //exit(0);
  
  mapSize = filesize+BUFFERSIZE;
  db("mapSize: "<<mapSize<<"\n");
  pos = filesize/sizeof(uint32_t); //in elements
  db("startPosSize: "<<pos<<"\n");

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

#ifdef DEBUG
void MainHeader::showData(int lineStart, int lineEnd){  
  std::cout<<"------------------------------\n";
  for(int i =lineStart*2; i<lineEnd*2; i+=2){
    std::cout<<"Tstamp: "<<data[i+0]<<"\n";
    std::cout<<"byteInDataFile: "<<data[i+1]<<"\n";
  }
}
#endif

void MainHeader::findFullTS(uint32_t Tstamp, int& A, int& B) {
  int midIdx;
  uint32_t element;
  
  int highIdx = pos;
  int lowIdx = 0;
  
  db("trying to find timestamp: "<<Tstamp<<"\n");
  
  while(highIdx > lowIdx){
    midIdx = (highIdx+lowIdx)/4 *2; //gives even number : timestamp
    element = data[midIdx];
    db("midIdx, element: "<<midIdx<<" ,"<<element<<"\n");
    
    if(data[lowIdx] == Tstamp){ A = data[lowIdx+1]; B = data[lowIdx+1]; db("HERE6"); return;}
    else if(element == Tstamp){ A = data[midIdx+1]; B = data[midIdx+1]; db("HERE4"); return;}
    
    else if(element > Tstamp){//als te hoog
      if(highIdx == midIdx){//als we bijna klaar zijn
        A = data[lowIdx+1];
        B = data[highIdx+1];
        db("HERE1");
        return;
      }
      highIdx = midIdx;//upper bound omlaag (naar huidig midde)
    }
    else{//we zitten te laag
      if(lowIdx == midIdx) {//we kunnen niet hoger
        A = data[lowIdx+1];
        B = -1;//there is no timestamp higher //NOTE changed from orgn.
        db("HERE5");
        return;}
      lowIdx = midIdx;//lower bound omhoog (naar huidig midde)
    }
  }
  A = -1;
  B = data[lowIdx+1]; //NOTE changed from orgn.
  db("HERE3");
  return;
}

uint32_t MainHeader::lastFullTS(){
  if(pos < 2){ return 0;}
  else{ return data[pos-2];}
}

uint32_t MainHeader::fullTSJustBefore(unsigned int byte){
  for(int i = pos-2; i >= 0; i-=2){
    if(data[i+1] <= byte){
      return data[i]; //return timestamp
    }
  }
  return -1;
}

void MainHeader::getNextFullTS(unsigned int byte, unsigned int& nextFullTSLoc, 
                               uint32_t& nextFullTS){
  for(int i = 0; i<= pos-2; i+=2){
    if(data[i+1] >= byte){
      nextFullTS = data[i]; //return timestamp
      nextFullTSLoc = data[i+1];
      return;
    }
  }
  byte = 0;
  return;
}

//used for testing
#ifdef TESTO //test object file defined
int main(){
  int A;
  int B;

  MainHeader header("test");

  for(int i = 0; i<200; i++){
    //header.append(1481034435+2*i,i);
  }
  header.showData(0,200);

  //header.findFullTS(1481034435+2*30, A, B);
  //std::cout<<"interval: "<<A<<", "<<B<<"\n";

  std::cout<<header.lastFullTS()<<"\n";
  return 0;
}
#endif
