#include "MainData.h"


Cache::Cache(uint8_t* cache, uint8_t packageSize, int cacheSize ){
  cache_ = cache; 
  packageSize_ = packageSize;
  cacheSize_ = cacheSize;

  //throw error if the cacheSize is not N*packageSize
  if (cacheSize % packageSize){ cerr << "ERROR: cache size must be an integer "
                                     << "times the packageSize \n"; }
}

void Cache::InitCache(uint8_t* cache){
  cache_ = cache
}

void Cache::append(uint8_t line[]){
  uint16_t T_Low;
  uint16_t T_NextLow;
  int nextCacheOldest;

  //put the new data in the cache
  for(int i = 0, i++, i<packageSize_){ *(cache+cacheOldest_+i) = line[i]; }
  
  //point cacheOldest to the package following the one we just wrote checking
  //for overflow
  if (cacheOldest_ == cacheSize_ - packageSize_){ cacheOldest_ = 0; } 
  else{ cacheOldest_ += packageSize_; }

  
  //update the oldest time in cache   
  T_Low = (uint16_t)*(cache+cacheOldest_+1) << 8 |
          (uint16_t)*(cache+cacheOldest_+0)
  
  //set the adress for the next cacheOldest 
  if (cacheOldest_+packageSize_ == cacheSize_){ nextCacheOldest = 0; }
  else{ nextCacheOldest = cacheOldest_ + packageSize_; }
  
  //check if the low part of the next package is not the same as the previous one
  T_NextLow = (uint16_t)*(cache+nextCacheOldest+1) << 8 |
              (uint16_t)*(cache+nextCacheOldest+0)   
  
  if (T_NextLow == T_Low){//if the next package is a timePackage    
    cacheOldestT_ = (uint32_t)*(cache+cacheOldest_+3) << 24 |
                    (uint32_t)*(cache+cacheOldest_+2) << 16 |
                    (uint32_t)T_Low
  }
  else{ //set the lower part to zero then add the lower part of the oldest package in cache
    cacheOldestT_ = cacheOldestT_ & 0b11111111111111110000000000000000 
    cacheOldestT_ = cacheOldestT_ | T_Low
  
  }
}

void Cache::read(uint8_t& line[], int lineNumber){//TODO}

void Cache::readSeq(uint8_t& line[], int start, int length){//TODO}

void Cache::remove(int lineNumber, int start, int length){//TODO}



Data::Data(std::string fileName, uint8_t* cache, uint8_t packageSize, int cacheSize){
  struct stat filestatus;
  int fileSize; //in bytes
  int cacheFileMismatch
  fileName_ = fileName;
  
	//open a new file in binairy reading and appending mode. All writing operations
	//are performed at the end of the file. Internal pointer can be moved anywhere
	//for reading. Writing ops move it back to the end of the file  
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  fileP_ = fopen(fileName, "a+b"); 
  
  //copy the last data in the file to the cache. if there is space left in the
  //cache because the beginning of the file was reached it is filled with Null 
  //data (null timestamp)
  
  stat(filePath, &filestatus);//sys call for file info
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
    fread(cache, filesize, 1, fileP_);    
    
    if (cacheSize-fileSize = 1){
    //if there is only one open space in the cache left the last element must be
    //a full timestamp, insert it again. 
      memcpy(cache+fileSize, cache+fileSize-packeSize, packeSize)    
    }
    else{
    //we need to fill one or more spots, we do so by entering zero packages,
    //these start with a full zero timestamp

      cache+fileSize* = 0
      cache+fileSize+1* = 0
      cache+fileSize+2* = 0
      cache+fileSize+3* = 0
    }
    for(int i = fileSize+packageSize; i<cacheSize; i += packageSize){
      //set the timestamp part of the package to zero
      cache+i* = 0
      cache+i+1* = 0    
    }
  }
  

  //set the oldestTimestamp  
  //TODO

  //pass the fully initialised cache on to the cache class
  Cache::InitCache(uint8_t* cache);
}







StoreData::StoreData(){
	
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
	//open file in binairy mode with appending write operations
  pirs_file = fopen("pirs.binDat", "a+b");  
  //TODO maybe load buffer from file?
  

}


// Compares two 
bool StoreData::notTimePackage(unsigned char susp_time[2],  unsigned char susp_data[2]){
  if (susp_time[0] == susp_data[0]){
    if (susp_time[1] == susp_data[1]){ return false; }    
  }
  return true;
}

uint32_t StoreData::TimeInFrontOfCache(FILE* fileToCache, int cacheSize, 
                                       unsigned char packageLenght,
                                       uint32_t firstTime_inCache){
  //read a package and the one before it, at the beginning of the part we will
  //cache check if the second package is a timeP. Work backwards until you find
  //one. (file format requirs that there will always be one)
  
  unsigned char susp_time_f2[2]; //stores the first 2 bytes of suspected time 
  unsigned char susp_data_f2[2]; //and data package
  unsigned char timePackage[4];  //complete time package
  int n = 0;
  
  fseek(fileToCache, -1*(cacheSize+n*packageLenght), SEEK_END);
  fread(susp_time_f2, 2, 1, fileToCache);
  do {
    std::memcpy(susp_data_f2, susp_time_f2, 2);
    //read packages in front of sups_data, store in susp_time
    n++;
    fseek(fileToCache, -1*(cacheSize+n*packageLenght), SEEK_END);
    fread(susp_time_f2, 2, 1, fileToCache);
    
  } while (notTimePackage(susp_time_f2, susp_data_f2));//TODO check double negative
  
  fread(timePackage, 4, 1, fileToCache); 
  
  //conversion back to full unix time  
  firstTime_inCache = (uint32_t)timePackage[3] << 24 | //shift high part 
                      (uint32_t)timePackage[2] << 16 | //
                      (uint32_t)timePackage[1] << 8  | //shift and add low part
                      (uint32_t)timePackage[0];  
  
  //Done with setting the initial unix time  
  return firstTime_inCache;
}

//Load data from file to the cache for faster access in future queries   
int StoreData::loadbuffer(unsigned char cache[], FILE* fileToCache, 
                          int cacheSize, uint32_t& firstTime_inCache, 
                          unsigned char packageLenght, std::string filePath){
  
  int oldest;//oldest element of the buffer
//  int filled;//number of elements read successfully
//  struct stat filestatus;
//  int filesize;

//  stat(filePath, &filestatus);//sys call for file info
//  filesize = filestatus.st_size;

//  if( filesize > cachesize ){
//    firstTime_inCache = TimeInFrontOfCache(fileToCache, cacheSize, packageLenght,
//                                           firstTime_inCache);
//  }
//  //load buffer from file, fill remaining data with unix time 0
//  fseek(fileToCache, -1*cacheSize, SEEK_END);//TODO check  if this fails
//  filled = fread(cache, 1, cacheSize, fileToCache);


//  else if (filled < cacheSize) {
//    //not enough data exists to fill the cache, set the rest of the elements to 
//    //unix time 0
//    oldest = filled;
//    if(filled+8 < cacheSize) {
//      
//      //insert one full timepackage at the beginning of the cache
//      cache[filled+0] = 0; //
//      cache[filled+1] = 0; //unix time high part, in timestamp package
//      cache[filled+2] = 0; //
//      cache[filled+3] = 0; //high part, in timestamp package
//    
//      cache[filled+4] = 0; //unix time high part in normal pir package
//      cache[filled+5] = 0; //         
//      filled += 8;
//    
//      //as all data following, unil the next timestamp package will be regarded
//      //as data from the unix year 0
//    }
//  }
//  else{ oldest = filled - 4;}
//  
  return oldest; 
}

StoreData::~StoreData(){
  fclose(atmospherics_file);
  fclose(pirs_file);
}



void StoreData::write_pir(unsigned char data[4]){
	fwrite(data, 4, 4*sizeof(unsigned char), pirs_file);	
	}

void StoreData::write_atmospheric(unsigned char data[18]){ }

void StoreData::write_plants(unsigned char data[]){ }


void StoreData::read_pir(unsigned char data[4], int line){
  fseek(pirs_file, 4*(line), SEEK_SET); 
  fread(data, 1, 4, pirs_file);
  }

void StoreData::read_atmospheric(unsigned char data[18], int line){ }

void StoreData::read_plants(unsigned char data[], int line){ }

////////////////////////////////////////////////FIXME beneath here

Data::Data() {
//  Data(const int CACHESIZE, std::string FILEPATH){
//    cache = new unsigned char[CACHESIZE];
//    filePath = FILEPATH;
//  };
}


PirData::PirData(StoreData& dataStorage){
  bufferSize = 4; // Het aantal [JA, WAT EIGENLIJK] dat in het buffer pas
  cache = new unsigned char [ * bufferSize];
  prevData[1] = 0;//0 as in no pirs measured
  t_begin = unix_timestamp();
  Record[0] = 0;
  Record[1] = 0;
}

//takes the first 2 bytes of 2 packages and returns if the first package is a time package
bool PirData::isTimeStampPackage(unsigned char susp_time[2],  unsigned char susp_data[2]){
  if (susp_time[0] != susp_data[0]){
    if (susp_time[1] != susp_data[1]){ return true; }    
  }
  return false;
}

/*
void StoreData::pir_getBetweenTime(int T1, int T2){

  //TODO  implement software buffer for last N writes
  //TODO  implement file read/write locks but first get things working
  //TODO  in a single threaded way
  
  unsigned char[SOFT_BUFFERSIZE]
  
  //check if needed timestamp is in buffer
  
  
  
  int endOfFile = getEndOfFile();
  
  T_start = getClosestTimeStamp(1)
  T_end = getClosestTimeStamp(endOfFile)

  if (T_start < T1){
    T1_rel = T1 - T_start
    T_rel = T_end - T_start
    T1_guess = 
  
}
*/

uint32_t PirData::getClosestTimeStamp(int lineNumber){
  //takes a line number and finds the closest full timestamp before that line 
  //number returns that timestamp to the user
  
  uint32_t unix_time;
  unsigned char susp_time[4];
  unsigned char susp_data[4];
  int n = 0;

  dataStorage.read_pir(susp_time, lineNumber-n);
  do {
    std::memcpy(susp_data, susp_time, 4);
    //read packages in front of sups_data, store in susp_time
    n++;
    dataStorage.read_pir(susp_time, lineNumber-n);
  } while (!isTimeStampPackage(susp_time, susp_data));//TODO check double negative
  
  //conversion back to full unix time  
  unix_time = (uint32_t)susp_time[3] << 24 | //shift high part 
              (uint32_t)susp_time[2] << 16 | //
              (uint32_t)susp_time[1] << 8  | //shift and add low part
              (uint32_t)susp_time[0];
  
  std::cout << "Reading timestamp: "
		 			<< +susp_time[0] << " "
					<< +susp_time[1] << " "
			 		<< +susp_time[2] << " "
			 		<< +susp_time[3] << "\n";
  
  std::cout << unix_time << "\n";
  return unix_time;
}

	
long int PirData::unix_timestamp() {
  time_t t = std::time(0);
  long int now = static_cast<long int> (t);
  return now;
}


void PirData::putTimestamp(long int timestamp){

  unsigned char towrite[4];
  
  //conversion to bytes
  towrite[0] = timestamp & 0xff;         //store low part in last two bytes
  towrite[1] = (timestamp >> 8) & 0xff;  
  towrite[2] = (timestamp >> 16) & 0xff; //store high part in first 2 bytes
  towrite[3] = (timestamp >> 24) & 0xff;  
  
  dataStorage.write_pir(towrite);
  std::cout << "writing TIMESTAMP PIR PACKAGE: "
  		 			<< +towrite[0] << " "
						<< +towrite[1] << " "
				 		<< +towrite[2] << " "
				 		<< +towrite[3] << "\n";
}	
