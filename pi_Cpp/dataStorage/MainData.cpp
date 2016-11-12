#include "MainData.h"


Cache::Cache(const uint8_t packageSize, const int cacheSize ){
  packageSize_ = packageSize;
  cacheSize_ = cacheSize;
  //as the cache is always fully filled on startup (with dummy data at the begin
  //if the file does not fill the complete cache set cacheOldest to 0
  cacheOldest_ = 0;

  //throw error if the cacheSize is not N*packageSize
  if (cacheSize % packageSize){ std::cerr << "ERROR: cache size must be an "
                                << "integer times the packageSize \n"; }
  if (packageSize > MAXPACKAGESIZE){ std::cerr << "ERROR: packageSize must be "
                                      << "smaller then: "<< MAXPACKAGESIZE <<" "
                                      << "try increasing 'MAXPACKAGESIZE'\n"; }
}

void Cache::InitCache(uint8_t* cache){
  cache_ = cache;
}

void Cache::append(const uint8_t line[]){
  uint16_t T_Low;
  uint16_t T_NextLow;
  int nextCacheOldest;

  std::cout<<"cache size: "<<cacheSize_<<" cache oldest: "<<cacheOldest_<<"\n";

  //put the new data in the cache
  for(int i = 0; i<packageSize_; i++){ *(cache_+cacheOldest_+i) = line[i]; }

  //point cacheOldest to the package following the one we just wrote checking
  //for overflow
  if (cacheOldest_ == cacheSize_ - packageSize_){ cacheOldest_ = 0; } 
  else{ cacheOldest_ += packageSize_; }
 
  //update the oldest time in cache   
  T_Low = (uint16_t) *(cache_+cacheOldest_+1) << 8 |
          (uint16_t) *(cache_+cacheOldest_+0);
  
  //set the adress for the next cacheOldest 
  if (cacheOldest_+packageSize_ == cacheSize_){ nextCacheOldest = 0; }
  else{ nextCacheOldest = cacheOldest_ + packageSize_; }
  
  //check if the low part of the next package is not the same as the previous one
  T_NextLow = (uint16_t) *(cache_+nextCacheOldest+1) << 8 |
              (uint16_t) *(cache_+nextCacheOldest+0);
  
  if (T_NextLow == T_Low){//if the next package has the same time low part 
    //then this package is a time package and must be treated as such    
    cacheOldestT_ = (uint32_t) *(cache_+cacheOldest_+3) << 24 |
                    (uint32_t) *(cache_+cacheOldest_+2) << 16 |
                    (uint32_t) T_Low;
  }
  else{ //set the lower part to zero then add the lower part of the oldest package in cache
    cacheOldestT_ = cacheOldestT_ & 0b11111111111111110000000000000000;
    cacheOldestT_ = cacheOldestT_ | T_Low;
  
  }
}

void Cache::read(uint8_t line[], int lineNumber){//TODO
  }

void Cache::readSeq(uint8_t line[], int start, int length){//TODO
  }

void Cache::remove(int lineNumber, int start, int length){//TODO
  }



Data::Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize)
  : Cache(packageSize, cacheSize)
{
  struct stat filestatus;
  int fileSize; //in bytes
  int n;
  
  uint8_t lineB[MAXPACKAGESIZE];
  uint8_t lineA[MAXPACKAGESIZE];
  
  /*set class variables*/
  fileName_ = "data/"+fileName;
  packageSize_ = packageSize;
  
	//open a new file in binairy reading and appending mode. All writing operations
	//are performed at the end of the file. Internal pointer can be moved anywhere
	//for reading. Writing ops move it back to the end of the file  
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  fileP_ = fopen(fileName_.c_str(), "a+b"); 
  
  std::cout<<"fileP_: "<< +fileP_<<"\n";
  
  //copy the last data in the file to the cache. if there is space left in the
  //cache because the beginning of the file was reached it is filled with Null 
  //data (null timestamp)
  
  stat(fileName.c_str(), &filestatus);//sys call for file info
  fileSize = filestatus.st_size;
  
  if (fileSize >= cacheSize){
    //set the file pointer to cachesize from the end of the file then
    //read from there to the end of the file into the cache
    fseek(fileP_, -1*(cacheSize), SEEK_END); 
    fread(cache, cacheSize, 1, fileP_);
  
    /*set the oldest timestamp in the cache*/
    //look through the file for a timestamp package starting at -cachesize from
    //the end

    n = -1*(cacheSize);
    fseek(fileP_, n, SEEK_END); 
    fread(lineB, packageSize, 1, fileP_); 
    n -= packageSize;
     
    fseek(fileP_, n, SEEK_END); 
    fread(lineA, packageSize, 1, fileP_);   
    n -= packageSize;
    
    while (notTSpackage(lineA, lineB)){    
    
      memcpy(lineB, lineA, packageSize);
       
      fseek(fileP_, n, SEEK_END); 
      fread(lineA, packageSize, 1, fileP_);   
      n -= packageSize;   
    }
    //save the timepackage
    cacheOldestT_ = (uint32_t)*(lineA+3) << 24 |
                    (uint32_t)*(lineA+2) << 16 |
                    (uint32_t)*(lineA+1) << 8  |
                    (uint32_t)*(lineA+0);
    
  }
  else{
    //set the file pointer to the beginning of the file then read in data till
    //the end of file. Next fill everything with 0 data.
    fseek(fileP_, 0, SEEK_SET);
    fread(cache+(cacheSize-fileSize), fileSize, 1, fileP_);//FIXME check 2e argument
    
    /*fill the remainder of the cache*/    
    if (cacheSize-fileSize == 1){
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
    /*set the oldest timestamp in the cache*/
    //as the complete file is in the cache and the file must start with a full
    //timestamp we can just convert the first package to a timestamp
    cacheOldestT_ = (uint32_t)*(cache+3) << 24 |
                    (uint32_t)*(cache+2) << 16 |
                    (uint32_t)*(cache+1) << 8  |
                    (uint32_t)*(cache+0);
  }
  initTimeStampNotSet = false;
  //pass the fully initialised cache on to the cache class
  Cache::InitCache(cache);
}

FILE* Data::getFileP(){
  return fileP_;
}

void Data::append(uint8_t line[], uint32_t Tstamp){
  std::cout << "enterd Data::append\n";
  uint8_t towrite[MAXPACKAGESIZE];
  uint16_t timeLow;
  
  //we need to put a full timestamp package in front of this package if
  //we have just started again, time < halfAday and we have not set the first
  //timestamp, or time > halfAday and we have not set the second timestamp.
  
  timeLow = static_cast<uint16_t>(Tstamp);
	
	if (initTimeStampNotSet){ 
	  putFullTS(Tstamp);//writes it to file and cache too  
	  initTimeStampNotSet=true;
  }	
	else if ((prevTstamp >> 16) != (Tstamp >> 16)) {
	  putFullTS(Tstamp);  
  }

  //put the unix time in front of the package  
  std::memcpy(towrite+2 , line, packageSize_-2);
  towrite[0] = timeLow | 0b1111111100000000;
  towrite[1] = timeLow | 0b0000000011111111;
  
  std::cout << "data hier1\n";
  
  Cache::append(towrite);//writes it to file and cache too  
  fwrite(towrite, 1, packageSize_, fileP_);
  std::cout << "leaving Data::append\n";
}

void Data::read(uint8_t line[], int lineNumber){//TODO
  }

void Data::readSeq(uint8_t line[], int start, int length){//TODO
  }

void Data::remove(int lineNumber, int start, int length){//TODO
  }

int Data::searchTstamp(uint32_t Tstamp, int startLine){//TODO
  return 0;
  }

void Data::putFullTS(const uint32_t Tstamp){
  uint8_t towrite[MAXPACKAGESIZE];
  prevTstamp = Tstamp;
  
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
