#include "MainData.h"


Data::Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize)
  : Cache(packageSize, cacheSize), MainHeader(fileName)
{
  struct stat filestatus;
  int fileSize; //in bytes
  
  /*set class variables*/
  fileName_ = "data/"+fileName+".binDat";
  packageSize_ = packageSize;
  
	//open a new file in binairy reading and appending mode. All writing operations
	//are performed at the end of the file. Internal pointer can be moved anywhere
	//for reading. Writing ops move it back to the end of the file  
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  fileP_ = fopen(fileName_.c_str(), "a+b");
  
  //copy the last data in the file to the cache. if there is space left in the
  //cache because the beginning of the file was reached it is filled with Null 
  //data (null timestamp)

  stat(fileName_.c_str(), &filestatus);//sys call for file info
  fileSize = filestatus.st_size;

  if (fileSize >= cacheSize){
    //set the file pointer to cachesize from the end of the file then
    //read from there to the end of the file into the cache
    fseek(fileP_, -1*(cacheSize), SEEK_END); 
    fread(cache, cacheSize, 1, fileP_);
  }
  else{
    //set the file pointer to the beginning of the file then read in data till
    //the end of file. Next fill everything with 0 data.
    fseek(fileP_, 0, SEEK_SET);
    fread(cache+(cacheSize-fileSize), fileSize, 1, fileP_);//FIXME check 2e argument
    
    /*fill the remainder of the cache*/    
    if (cacheSize-fileSize == 1){
      //FIXME this wont work
    //if there is only one open space in the cache left the last element must be
    //a full timestamp, insert it again. 
      memcpy(cache+fileSize, cache+fileSize-packageSize_, packageSize_);  
    }
    else{
    //we need to fill one or more spots, we do so by entering zero packages,
    //these start with a full zero timestamp

      *(cache+fileSize) = 0;    //cache is the memory adress where the cache is at
      *(cache+fileSize+1) = 0;
      *(cache+fileSize+2) = 0;
      *(cache+fileSize+3) = 0;
    }
    for(int i = fileSize+packageSize_; i<cacheSize; i += packageSize_){
      //set the timestamp part of the package to zero
      *(cache+i) = 0;
      *(cache+i+1) = 0;
    }
  }
  prevFTstamp = MainHeader::lastFullTS();
  //pass the fully initialised cache on to the cache class
  Cache::InitCache(cache);
}

FILE* Data::getFileP(){
  return fileP_;
}

void Data::append(uint8_t line[], uint32_t Tstamp){
  uint8_t towrite[MAXPACKAGESIZE];
  uint16_t timeLow;
  
  //we need to put a full timestamp package in front of this package if
  //we have just started again, time < halfAday and we have not set the first
  //timestamp, or time > halfAday and we have not set the second timestamp.

  std::cout<<"prevFTstamp: "<<prevFTstamp<<" Tstamp: "<<Tstamp<<"\n";
	if (prevFTstamp >> 16 != Tstamp >> 16) {
	  putFullTS(Tstamp);
  }

  timeLow = static_cast<uint16_t>(Tstamp);

  //put the unix time in front of the package  
  std::memcpy(towrite+2 , line, packageSize_-2);
  towrite[0] = timeLow | 0b1111111100000000;
  towrite[1] = timeLow | 0b0000000011111111;
  
  Cache::append(towrite);//writes it to file and cache too  
  fwrite(towrite, 1, packageSize_, fileP_);
}

void Data::readSeq(uint8_t line[], int start, int length){//TODO
  }

void Data::remove(int lineNumber, int start, int length){//TODO
  }

void Data::putFullTS(const uint32_t Tstamp){
  std::cout<<"putting full timestamp\n";
  uint8_t towrite[MAXPACKAGESIZE]; //towrite to data
  int currentByte;
  prevFTstamp = Tstamp;
  
  //copy the full timestamp to the start of the package
  uint8_t *p = (uint8_t*)&Tstamp;
  towrite[0] = p[0];
  towrite[1] = p[1];
  towrite[2] = p[2];
  towrite[3] = p[3];
  
  //fill up the rest of the package with zeros
  for (int i = 4; i<packageSize_; i++){
    towrite[i] = 0;
  }
  Cache::append(towrite);
  fwrite(towrite, 1, packageSize_, fileP_);
  currentByte = ftell(fileP_);
  MainHeader::append(Tstamp, currentByte);
}

bool Data::notTSpackage(uint8_t lineA[], uint8_t lineB[]){
  if (lineA[0] == lineB[0]){
    if (lineA[1] == lineB[1]){ return false; }    
  }
  return true;
}

uint32_t Data::unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

//SEARCH FUNCT
void Data::searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2, unsigned int& loc1, unsigned int& loc2) {
  unsigned int startSearch;
  unsigned int stopSearch;
  unsigned int firstInCachTime;
  unsigned int fileSize;

  // check the full timestamp file to get the location of the full timestamp still smaller then
  // but closest toTstamp and the next full timestamp (that is too large thus). No need to catch the case
  // where the Full timestamp afther Tstamp does not exist as such a Tstamp would result into seaching in cache.
  MainHeader::findFullTS(Tstamp1, startSearch, stopSearch);

  fseek (fileP_, 0, SEEK_END);
  fileSize = ftell (fileP_);

  firstInCachTime = MainHeader::fullTSJustBefore(fileSize - Cache::cacheSize_);
  firstInCachTime = (firstInCachTime & 0b11111111111111110000000000000000) | Cache::getFirstLowTime();

  //check if the wanted timestamp could be in the cache
  std::cout<<"Tstamp1: "<<Tstamp1<<" Data::cacheOldestT_: "<<firstInCachTime<<"\n";
  if (Tstamp1 > firstInCachTime){
    loc1 = findTimestamp_inCache(Tstamp1, startSearch, stopSearch, fileSize);
  }
  else{
    loc1 = findTimestamp_inFile(Tstamp1, startSearch, stopSearch);
  }

  MainHeader::findFullTS(Tstamp2, startSearch, stopSearch);

  if (Tstamp2 > firstInCachTime){
    loc2 = findTimestamp_inCache(Tstamp2, startSearch, stopSearch, fileSize);
  }
  else{
    loc2 = findTimestamp_inFile(Tstamp2, startSearch, stopSearch);
  }
  std::cout<<"loc1: "<<loc1<<"\tloc2: "<<loc2<<"\n";
}

int Data::findTimestamp_inFile(uint32_t Tstamp, unsigned int startSearch, unsigned int stopSearch){
  uint8_t block[MAXBLOCKSIZE];
  uint16_t Tstamplow;
  int Idx;

  Tstamplow = Tstamp & 0b00000000000000001111111111111111;

  // find maximum block size, either the max that fits N packages, or the space between stop and start
  unsigned int blockSize = std::min(MAXBLOCKSIZE-(MAXBLOCKSIZE%packageSize_), stopSearch-startSearch);

  Idx = -1;
  fseek(fileP_, startSearch, SEEK_SET);
  do {
    fread(block, blockSize, 1, fileP_); //read everything into the buffer

    Idx = searchBlock(block, Tstamplow, blockSize);
    // check through the block for a timestamp
  } while(Idx == -1);//this could go on forever but luckily we made our file correctly... right??? right??!!

  return Idx+startSearch;
}

int Data::searchBlock(uint8_t block[], uint16_t Tstamplow, unsigned int blockSize) {
  uint16_t timelow;

  for(unsigned int i = 0; i<blockSize; i+=packageSize_){
    timelow = (uint16_t)block[i+1] << 8  |
              (uint16_t)block[i];
    if(timelow > Tstamplow){//then
      return i-packageSize_;
    }
  }
  return -1;
}

int Data::findTimestamp_inCache(uint32_t Tstamp, unsigned int startSearch, unsigned int stopSearch, unsigned int fileSize){
  unsigned int startInCache;
  unsigned int stopInCache;

  //TODO SOMETHING WRONG HERE
  std::cout<<"startSearch: "<<startSearch<<" cacheSize: "<<Cache::cacheSize_<<"fileSize: "<<fileSize<<"\n";
  startInCache = startSearch + Cache::cacheSize_ - fileSize;
  stopInCache = stopSearch + Cache::cacheSize_ - fileSize;

  return Cache::searchTimestamp(Tstamp, startInCache, stopInCache);
}
