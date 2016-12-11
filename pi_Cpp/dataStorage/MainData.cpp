#include "MainData.h"
#include "../graph/MainGraph.h" //only needed for MAXPLOTRESOLUTION

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

  //std::cout<<"prevFTstamp: "<<prevFTstamp<<" Tstamp: "<<Tstamp<<"\n";
	if (prevFTstamp >> 16 != Tstamp >> 16) {
	  putFullTS(Tstamp);
  }

  timeLow = static_cast<uint16_t>(Tstamp);
  std::cout<<"timeLow: "<<timeLow<<"\n";

  //put the unix time in front of the package  
  std::memcpy(towrite+2 , line, packageSize_-2);
  uint8_t *p = (uint8_t*)&timeLow;
  towrite[0] = p[0];
  towrite[1] = p[1];
  //towrite[1] = (uint8_t)timeLow | 0b1111111100000000;//old method
  //towrite[0] = (uint8_t)timeLow | 0b0000000011111111;
  std::cout<<"towrite[1]: "<<+towrite[1];
  std::cout<<"towrite[0]: "<<+towrite[0]<<"\n";
  
  Cache::append(towrite);//writes it to file and cache too  
  fwrite(towrite, 1, packageSize_, fileP_);
}

#ifdef DEBUG
void Data::showLines(int start_P, int end_P){
  uint8_t packageBegin[4];
  uint16_t timeLow;
  uint32_t TimeBegin;
  uint32_t FullTime;
  TimeBegin = MainHeader::fullTSJustBefore(start_P*packageSize_);
  std::cout<<"TimeBegin: "<<TimeBegin<<"\n";
  
  for( int i = start_P*packageSize_; i<end_P*packageSize_; i+=packageSize_){
    fseek(fileP_, i, SEEK_SET);
    fread(packageBegin, 1, packageSize_, fileP_);

    std::cout<<"read towrite[1]: "<<+packageBegin[1];
    std::cout<<"\tread towrite[0]: "<<+packageBegin[0];
    
    timeLow = packageBegin[1] << 8 |
              packageBegin[0];
    
    FullTime = (TimeBegin & 0b11111111111111110000000000000000) | timeLow;
    
    std::cout << "\tFullTime: "<<FullTime<<"\n";
  }
}
#endif

void Data::fetchData(uint32_t startT, uint32_t stopT, uint32_t x[], float y[],
                     float (*func)(int orgIdx_B, int blockIdx_B, uint8_t[MAXBLOCKSIZE],
                     int extraParams[4]), int extraParams[4]) {

  unsigned int startByte; //start position in the file
  unsigned int stopByte; //stop position in the file
  
  unsigned int nBlocks;
  unsigned int blockSize_B;
  unsigned int blockSize_P;
  unsigned int blockSize_bins;

  unsigned int rest_B;
//  unsigned int rest_P;
  unsigned int rest_bin;

  unsigned int binSize_P;
  unsigned int binSize_B;

  unsigned int binNumber;
  unsigned int orgIdx_P;
  unsigned int orgIdx_B;
  unsigned int blockIdx_B;

  uint8_t block[MAXBLOCKSIZE];

  //find where to start and stop reading in the file
  searchTstamps(startT, stopT, startByte, stopByte);
  initGetTime(startByte);

  //configure iterator
  iterator checkIdx(startByte, stopByte, packageSize_);
  binSize_P = checkIdx.indexGroupSize; //number of packages in a bin
  binSize_B = binSize_P * packageSize_; //number of bytes in a bin

  //set subarrays for binning
  uint32_t* x_bin = new uint32_t[binSize_P]; //used to store time values in when binning
  float* y_bin = new float[binSize_P]; //used to store the y value of whatever we want to know in

  //set number of blocks, blocksize, and the rest bit's size
  nBlocks = (stopByte - startByte)/MAXBLOCKSIZE; //calculate how many blocks we need

  blockSize_B = MAXBLOCKSIZE - (MAXBLOCKSIZE%packageSize_); //determine blocksize in bytes
  blockSize_P = blockSize_B/packageSize_; //set blocksize in packages
  blockSize_bins = blockSize_B/binSize_B; //set blocksize in bins

  rest_B = (stopByte=startByte)%MAXBLOCKSIZE; //number of bytes that doesnt fit in the normal blocks
//  rest_P = rest_B/blockSize_P; //in packages
  rest_bin =rest_B/binSize_B; //tobin

  //iterate over the blocks
  for (unsigned int i = 0; i < nBlocks; i++) {
    //read one block to memory
    fseek(fileP_, startByte+i*blockSize_B, SEEK_SET);
    fread(block, 1, blockSize_B, fileP_);

    //iterate through the block in memory in bin groups
    for (unsigned int j = 0; j < blockSize_bins; j++) {
      binNumber = i*blockSize_bins +j; //keep track which bin we are calculating

      //iterate through a group of values to bin
      for (unsigned int k = 0; k < binSize_P; k ++) {
        orgIdx_P = i*blockSize_P+ j*binSize_P;
        if (checkIdx.useValue(orgIdx_P)) {
          orgIdx_B = orgIdx_P* packageSize_;
          blockIdx_B = j*binSize_B+ k*packageSize_;

          x_bin[k] = getTime(orgIdx_B, blockIdx_B, block);
          y_bin[k] = func(orgIdx_B, blockIdx_B, block, extraParams);
        }
      }
      y[binNumber] = mean(x_bin, binSize_B);
      x[binNumber] = mean(y_bin, binSize_B);
    }
  }

  //do the leftover values in a smaller block
  fseek(fileP_, stopByte-rest_B, SEEK_SET);
  fread(block, 1, rest_B, fileP_);

  //iterate through the block in memory in bin groups
  for (unsigned int j = 0; j < rest_bin; j++) {
    binNumber = nBlocks*blockSize_bins +j;

    //iterate through a group of values to bin
    for (unsigned int k = 0; k < binSize_P; k ++) {
      orgIdx_P = nBlocks*blockSize_P+ j*binSize_P;
      if (checkIdx.useValue(orgIdx_P)) {
        orgIdx_B = orgIdx_P* packageSize_;
        blockIdx_B = j*binSize_B+ k*packageSize_;

        x_bin[k] = getTime(orgIdx_B, blockIdx_B, block);
        y_bin[k] = func(orgIdx_B, blockIdx_B, block, extraParams);
      }
    }
    y[binNumber] = mean(x_bin, binSize_B);
    x[binNumber] = mean(y_bin, binSize_B);
  }

}//done

void Data::remove(int lineNumber, int start, int length){//TODO
  }

//SEARCH FUNCT
void Data::searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2, unsigned int& loc1, unsigned int& loc2) {
  unsigned int startSearch;
  unsigned int stopSearch;
  unsigned int firstInCachTime;
  unsigned int fileSize;

  fseek (fileP_, 0, SEEK_END);
  fileSize = ftell (fileP_);

  // check the full timestamp file to get the location of the full timestamp still smaller then
  // but closest toTstamp and the next full timestamp (that is too large thus). No need to catch the case
  // where the Full timestamp afther Tstamp does not exist as such a Tstamp would result into seaching in cache.
  MainHeader::findFullTS(Tstamp1, startSearch, stopSearch);
  if(stopSearch == 0){stopSearch = fileSize;}

  firstInCachTime = MainHeader::fullTSJustBefore(fileSize - Cache::cacheSize_);
  firstInCachTime = (firstInCachTime & 0b11111111111111110000000000000000) | Cache::getFirstLowTime();

  //check if the wanted timestamp could be in the cache
  //std::cout<<"Tstamp1: "<<Tstamp1<<" Data::cacheOldestT_: "<<firstInCachTime<<"\n";
  if (Tstamp1 > firstInCachTime){
    loc1 = findTimestamp_inCache(Tstamp1, startSearch, stopSearch, fileSize);
  }
  else{
    loc1 = findTimestamp_inFile(Tstamp1, startSearch, stopSearch);
  }

  MainHeader::findFullTS(Tstamp2, startSearch, stopSearch);
  if(stopSearch == 0){stopSearch = fileSize;}

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
  int counter = -1;
  int Idx;

  Tstamplow = (uint16_t)Tstamp;

  // find maximum block size, either the max that fits N packages, or the space between stop and start
  unsigned int blockSize = std::min(MAXBLOCKSIZE-(MAXBLOCKSIZE%packageSize_), stopSearch-startSearch);
  std::cout<<"s-s"<<(stopSearch-startSearch)<<"\n";

  Idx = -1;
  fseek(fileP_, startSearch, SEEK_SET);
  do {
    fread(block, blockSize, 1, fileP_); //read everything into the buffer
    counter++;
    Idx = searchBlock(block, Tstamplow, blockSize);
    // check through the block for a timestamp
  } while(Idx == -1);//this could go on forever but luckily we made our file correctly... right??? right??!!

  return Idx+startSearch+counter*blockSize;
}

int Data::searchBlock(uint8_t block[], uint16_t Tstamplow, unsigned int blockSize) {
  uint16_t timelow;
  //std::cout<<"want timelow: "<<Tstamplow<<"\n";
  //std::cout<<"want blockSize: "<<blockSize<<"\n";
  for(unsigned int i = 0; i<blockSize; i+=packageSize_){
    timelow = (uint16_t)block[i+1] << 8  |
              (uint16_t)block[i];
    //std::cout<<"found timelow: "<<timelow<<"\n";
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

//DATAFETCH FUNCT
Data::iterator::iterator(unsigned int startByte, unsigned int stopByte, unsigned int packageSize){//TODO implement ignoring extra datapoints
  unsigned int numbOfValues = (stopByte-startByte)/packageSize;
  unsigned int numbUnusable = numbOfValues%MAXPLOTRESOLUTION;
  indexGroupSize = numbOfValues/MAXPLOTRESOLUTION;
  spacing = numbOfValues/numbUnusable;
  counter = 0;
}

bool Data::iterator::useValue(unsigned int i){
  //calculate if element 'i' should be used or not
  if(i == (unsigned int)(counter*spacing)){
    counter++;
    return false;
  }
  else{return true;}
}

void Data::initGetTime(int startByte){
  timeHigh = MainHeader::fullTSJustBefore(startByte);
  prevTimePart[0] = 0;
  prevTimePart[1] = 0;
}
 
uint32_t Data::getTime(int orgIdx_B, int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  uint16_t timelow;
  uint32_t fullTimeStamp;
  if(prevTimePart[0] == block[blockIdx_B] && prevTimePart[1] == block[blockIdx_B+1]){
    //calculate the full timestamp contained in prevTimePart
    timeHigh = 0 | (uint32_t)prevTimePart[3] << 24 |
                   (uint32_t)prevTimePart[2] << 16;
  }
  memcpy(prevTimePart, block+blockIdx_B, 4);//save the time part for comparing to the next block

  timelow = (uint16_t)block[blockIdx_B+1] << 8  |
            (uint16_t)block[blockIdx_B];
  fullTimeStamp = timeHigh | timelow;
  return fullTimeStamp;
}

uint32_t Data::mean(uint32_t* array, int len){
  uint32_t Mean;
  for(int i =0; i<len; i++){
    Mean+=*(array+i);
  }
  Mean /= len;
  return Mean;
}


float Data::mean(float* array, int len){
  uint32_t Mean;
  for(int i =0; i<len; i++){
    Mean+=*(array+i);
  }
  Mean /= len;
  return Mean;
}



//HELPER FUNCT
void Data::putFullTS(const uint32_t Tstamp){
  //std::cout<<"putting full timestamp\n";
  uint8_t towrite[MAXPACKAGESIZE]; //towrite to data
  int currentByte;
  prevFTstamp = Tstamp;

  //copy the full timestamp to the start of the package
  uint8_t *p = (uint8_t*)&Tstamp;
  towrite[0] = p[0];
  towrite[1] = p[1];
  towrite[2] = p[2];
  towrite[3] = p[3];
  
  std::cout<<"fullTP :"<<+towrite[0]<<", "<<+towrite[1]<<"\n";
  std::cout<<"fullTP :"<<+towrite[2]<<", "<<+towrite[3]<<"\n";

  //fill up the rest of the package with zeros
  for (int i = 4; i<packageSize_; i++){
    towrite[i] = 0;
  }
  Cache::append(towrite);
  fwrite(towrite, 1, packageSize_, fileP_);
  currentByte = ftell(fileP_)-packageSize_;
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
