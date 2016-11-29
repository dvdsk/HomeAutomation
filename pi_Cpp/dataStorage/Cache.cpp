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
