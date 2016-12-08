#include "Cache.h"


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

  //put the new data in the cache
  for(int i = 0; i<packageSize_; i++){ *(cache_+cacheOldest_+i) = line[i]; }

  //point cacheOldest to the package following the one we just wrote checking
  //for overflow
  if (cacheOldest_ == cacheSize_ - packageSize_){ cacheOldest_ = 0; } 
  else{ cacheOldest_ += packageSize_; }

}

void Cache::readSeq(uint8_t line[], int start, int length){//TODO
  }

void Cache::remove(int lineNumber, int start, int length){//TODO
  }

uint16_t Cache::getFirstLowTime(){
  uint16_t T_low;
  T_low = (uint16_t) *(cache_+cacheOldest_+1) << 8 |
          (uint16_t) *(cache_+cacheOldest_+0);
  return T_low;
}

int Cache::searchTimestamp(uint32_t Tstamp, int start, int stop) {
  std::cout<<"searching in cache, start: "<<start<<" stop: "<<stop<<"\n";
  uint16_t T_low;
  for(int i=start; i <= stop; i++){
    T_low = (uint16_t) *(cache_+cacheOldest_+1) << 8 |
            (uint16_t) *(cache_+cacheOldest_+0);
    if( (uint16_t)Tstamp == T_low){return i;}
  }
  return -1;
}