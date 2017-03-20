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

void Data::append(const uint8_t line[], const uint32_t Tstamp){
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
  //std::cout<<"timeLow: "<<timeLow<<"\n";

  //put the unix time in front of the package  
  std::memcpy(towrite+2 , line, packageSize_-2);
  uint8_t *p = (uint8_t*)&timeLow;
  towrite[0] = p[0];
  towrite[1] = p[1];
  //towrite[1] = (uint8_t)timeLow | 0b1111111100000000;//old method
  //towrite[0] = (uint8_t)timeLow | 0b0000000011111111;
  //std::cout<<"towrite[1]: "<<+towrite[1];
  //std::cout<<"towrite[0]: "<<+towrite[0]<<"\n";
  
  Cache::append(towrite);//writes it to file and cache too  
  fwrite(towrite, 1, packageSize_, fileP_);
}


void Data::showLines(int start_P, int end_P){
  uint8_t package[200];
  uint32_t timeLow;
  uint32_t TimeBegin;
  uint32_t FullTime;
  TimeBegin = MainHeader::fullTSJustBefore(start_P);
  std::cout<<"TimeBegin: "<<TimeBegin<<"\n";
  
  for( int i = start_P; i<end_P; i+=packageSize_){
    fseek(fileP_, i, SEEK_SET);
    fread(package, 1, packageSize_, fileP_);

	std::cout<<"byte: "<<i;
    std::cout<<"  package: ";
    for(int i = 0; i<packageSize_; i++){
      std::cout<<+package[i]<<",\t";
    }
    
    timeLow = package[1] << 8 |
              package[0];
    
    FullTime = (TimeBegin & 0b11111111111111110000000000000000) | timeLow;
    
    std::cout << "\tFullTime: "<<FullTime<<"\n";
  }
}


int Data::fetchBinData(uint32_t startT, uint32_t stopT, double x[], uint16_t y[],
                       uint16_t (*func)(int blockIdx_B, uint8_t[MAXBLOCKSIZE])) {

  int len = 0; //Length of y
  unsigned int startByte; //start position in the file
  unsigned int stopByte; //stop position in the file
  
  unsigned int nBlocks;
  unsigned int blockSize_B;
  unsigned int rest_B;

  unsigned int binSize_P;
	unsigned int binIdx_P = 0;

  unsigned int binNumber = 0;
  unsigned int orgIdx_P = 0;
  unsigned int orgIdx_B = 0;

  unsigned int nextFullTSLoc;
  uint32_t nextFullTS;

  uint8_t block[MAXBLOCKSIZE];

  //find where to start and stop reading in the file
  //std::cout<<"searching for ya timestamps\n";
  searchTstamps(startT, stopT, startByte, stopByte);
  MainHeader::getNextFullTS(startByte, nextFullTSLoc, nextFullTS);
	/*do a quick sanity check on the search results*/
	if(stopByte < startByte){std::cout<<"ERROR STOPBYTE<STARTBYTE\n";}

  std::cout<<"reading in data between: "<<startByte<<" and "<<stopByte<<" bytes\n";
  initGetTime(startByte);

  //configure iterator
  iterator checkIdx(startByte, stopByte, packageSize_);
  binSize_P = checkIdx.binSize_P; //number of packages in a bin

  //set subarrays for binning
  uint32_t* x_bin = new uint32_t[binSize_P]; //used to store time values in when binning
  uint16_t* y_bin = new uint16_t[binSize_P]; //used to store the y value of whatever we want to know

  //calculate how many blocks we need
  nBlocks = (stopByte - startByte)/MAXBLOCKSIZE;
  
  //determine blocksize in bytes
  blockSize_B = std::min(MAXBLOCKSIZE - (MAXBLOCKSIZE%packageSize_), stopByte-startByte); 

  rest_B = (stopByte-startByte)%MAXBLOCKSIZE; //number of bytes that doesnt fit in the normal blocks
  
  for (unsigned int i = 0; i < nBlocks; i++) {
    fseek(fileP_, startByte+i*blockSize_B, SEEK_SET);
    fread(block, 1, blockSize_B, fileP_); //read one block to memory

		unsigned int blockIdx_B =0;
		while(blockIdx_B <= blockSize_B){				

			if (checkIdx.useValue(orgIdx_P)){
				
				/*before we process this value check if we should do anything to get rdy*/
				if(orgIdx_B >= nextFullTSLoc){//update timehigh
		      timeHigh = nextFullTS & 0b11111111111111110000000000000000;
		      MainHeader::getNextFullTS(orgIdx_B+packageSize_, nextFullTSLoc, nextFullTS);            
		    }
				if(binIdx_P == binSize_P){//save bin result to array if at bincapacity
					binIdx_P = 0;
					x[binNumber] = meanT(x_bin, binSize_P);
		      y[binNumber] = meanB(y_bin, binSize_P);
					binNumber++;
					len++;
				}

				/*retrieve data and store for binning*/
        x_bin[binIdx_P] = getTime(blockIdx_B, block);
        y_bin[binIdx_P] = func(blockIdx_B, block);						
				binIdx_P++;
			}		

			/*always update counters*/
			orgIdx_P++;
			orgIdx_B += packageSize_;
			blockIdx_B += packageSize_;		
		}
		/*clean up, (save current bin even if its not filled)*/
		x[binNumber] = meanT(x_bin, binIdx_P+1);
    y[binNumber] = meanB(y_bin, binIdx_P+1); 
		binIdx_P = 0;	
	}

	/*read the remaining data into a smaller block*/
  fseek(fileP_, startByte+nBlocks*blockSize_B, SEEK_SET);
  fread(block, 1, rest_B, fileP_);

	unsigned int blockIdx_B =0;
	while(blockIdx_B <= blockSize_B){				

		if (checkIdx.useValue(orgIdx_P)){
			
			/*before we process this value check if we should do anything to get rdy*/
			if(orgIdx_B >= nextFullTSLoc){//update timehigh
	      timeHigh = nextFullTS & 0b11111111111111110000000000000000;
	      MainHeader::getNextFullTS(orgIdx_B+packageSize_, nextFullTSLoc, nextFullTS);            
	    }
			if(binIdx_P == binSize_P){//save bin result to array if at bincapacity
				binIdx_P = 0;
				x[binNumber] = meanT(x_bin, binSize_P);
	      y[binNumber] = meanB(y_bin, binSize_P);
				binNumber++;
				len++;
			}

			/*retrieve data and store for binning*/
      x_bin[binIdx_P] = getTime(blockIdx_B, block);
      y_bin[binIdx_P] = func(blockIdx_B, block);						
			binIdx_P++;
		}		

		/*always update counters*/
		orgIdx_P++;
		orgIdx_B += packageSize_;
		blockIdx_B += packageSize_;		
	}
	/*clean up, (save current bin even if its not filled)*/
	x[binNumber] = meanT(x_bin, binIdx_P+1);
  y[binNumber] = meanB(y_bin, binIdx_P+1); 
	binIdx_P = 0;	

  return len;
}//done

int Data::fetchData(uint32_t startT, uint32_t stopT, double x[], double y[],
                    uint16_t (*func)(int blockIdx_B, uint8_t[MAXBLOCKSIZE]), 
										double (*func2)(uint16_t integer_var)) {

  int len = 0; //Length of y
  unsigned int startByte; //start position in the file
  unsigned int stopByte; //stop position in the file
  
  unsigned int nBlocks;
  unsigned int blockSize_B;
  unsigned int rest_B;

  unsigned int binSize_P;
	unsigned int binIdx_P = 0;

  unsigned int binNumber = 0;
  unsigned int orgIdx_P = 0;
  unsigned int orgIdx_B = 0;

  unsigned int nextFullTSLoc;
  uint32_t nextFullTS;

  uint8_t block[MAXBLOCKSIZE];

  //find where to start and stop reading in the file
  //std::cout<<"searching for ya timestamps\n";
  searchTstamps(startT, stopT, startByte, stopByte);
  MainHeader::getNextFullTS(startByte, nextFullTSLoc, nextFullTS);
	/*do a quick sanity check on the search results*/
	if(stopByte < startByte){std::cout<<"ERROR STOPBYTE<STARTBYTE\n"; while(1);}


  initGetTime(startByte);

  //configure iterator
  iterator checkIdx(startByte, stopByte, packageSize_);
  binSize_P = checkIdx.binSize_P; //number of packages in a bin

  //set subarrays for binning
  uint32_t* x_bin = new uint32_t[binSize_P]; //used to store time values in when binning
  uint16_t* y_bin = new uint16_t[binSize_P]; //used to store the y value of whatever we want to know

  //calculate how many blocks we need
  nBlocks = (stopByte - startByte)/MAXBLOCKSIZE;
  
  //determine blocksize in bytes
  blockSize_B = std::min(MAXBLOCKSIZE - (MAXBLOCKSIZE%packageSize_), stopByte-startByte); 

  rest_B = (stopByte-startByte)%MAXBLOCKSIZE; //number of bytes that doesnt fit in the normal blocks

  std::cout<<"\nreading in data between: "<<startByte<<" and "<<stopByte<<" bytes\n";
	std::cout<<"numb of packages: "<<(stopByte-startByte)/packageSize_<<"\n";
	std::cout<<"binSize_P: "<<binSize_P<<"\n";
  
  for (unsigned int i = 0; i < nBlocks; i++) {
    fseek(fileP_, startByte+i*blockSize_B, SEEK_SET);
    fread(block, 1, blockSize_B, fileP_); //read one block to memory

		unsigned int blockIdx_B =0;
		while(blockIdx_B < blockSize_B){				

			if (checkIdx.useValue(orgIdx_P)){
				
				/*before we process this value check if we should do anything to get rdy*/
				if(orgIdx_B >= nextFullTSLoc){//update timehigh
		      timeHigh = nextFullTS & 0b11111111111111110000000000000000;
		      MainHeader::getNextFullTS(orgIdx_B+packageSize_, nextFullTSLoc, nextFullTS);            
		    }
				if(binIdx_P == binSize_P){//save bin result to array if at bincapacity
					binIdx_P = 0;

					x[binNumber] = meanT(x_bin, binSize_P);
		      y[binNumber] = func2(meanB(y_bin, binSize_P));
					binNumber++;
					len++;
				}

				/*retrieve data and store for binning*/
        x_bin[binIdx_P] = getTime(blockIdx_B, block);
        y_bin[binIdx_P] = func(blockIdx_B, block);						
				binIdx_P++;
			}		

			/*always update counters*/
			orgIdx_P++;
			orgIdx_B += packageSize_;
			blockIdx_B += packageSize_;		
		}
		/*clean up, (save current bin even if its not filled)*/
		x[binNumber] = meanT(x_bin, binIdx_P+1);
    y[binNumber] = func2(meanB(y_bin, binIdx_P+1) ); 
		binIdx_P = 0;	
	}

	/*read the remaining data into a smaller block*/
  fseek(fileP_, startByte+nBlocks*blockSize_B, SEEK_SET);
  fread(block, 1, rest_B, fileP_);

	unsigned int blockIdx_B =0;
	while(blockIdx_B <= blockSize_B){				

		if (checkIdx.useValue(orgIdx_P)){
			
			/*before we process this value check if we should do anything to get rdy*/
			if(orgIdx_B >= nextFullTSLoc){//update timehigh
	      timeHigh = nextFullTS & 0b11111111111111110000000000000000;
	      MainHeader::getNextFullTS(orgIdx_B+packageSize_, nextFullTSLoc, nextFullTS);            
	    }
			if(binIdx_P == binSize_P){//save bin result to array if at bincapacity
				binIdx_P = 0;
				x[binNumber] = meanT(x_bin, binSize_P);
	      y[binNumber] = func2(meanB(y_bin, binSize_P));
				binNumber++;
				len++;
			}

			/*retrieve data and store for binning*/
      x_bin[binIdx_P] = getTime(blockIdx_B, block);
      y_bin[binIdx_P] = func(blockIdx_B, block);						
			binIdx_P++;
		}		

		/*always update counters*/
		orgIdx_P++;
		orgIdx_B += packageSize_;
		blockIdx_B += packageSize_;		
	}
	/*clean up, (save current bin even if its not filled)*/
	x[binNumber] = meanT(x_bin, binIdx_P+1);
  y[binNumber] = func2( meanB(y_bin, binIdx_P+1)); 
	binIdx_P = 0;	

  return len;
}//done

void Data::remove(int lineNumber, int start, int length){//TODO
  }

//SEARCH FUNCT
void Data::searchTstamps(uint32_t Tstamp1, uint32_t Tstamp2, unsigned int& loc1, unsigned int& loc2) {
  int startSearch;
  int stopSearch;
  unsigned int firstInCachTime;
  unsigned int fileSize;

	/*do a quick sanity check on the paramaters*/
	if(Tstamp1 > Tstamp2){std::cout<<"ERROR TSTAMP1 > TSTAMP2\n"; while(1);}

  fseek (fileP_, 0, SEEK_END);
  fileSize = ftell (fileP_);

  // check the full timestamp file to get the location of the full timestamp 
  // still smaller then but closest toTstamp and the next full timestamp (that is
  // too large thus). No need to catch the case where the Full timestamp afther
  // Tstamp does not exist as such a Tstamp would result into seaching in cache.
  
  MainHeader::findFullTS(Tstamp1, startSearch, stopSearch);
  if(stopSearch == -1){stopSearch = fileSize; } //handle case Tstamp1 > last full TS   

  //std::cout<<"TS 1 startSearch: "<<startSearch<<" stopSearch: "<<stopSearch<<"\n";  
  //if(startSearch == -1){
    ////the searched timestamp is earier then the earliest we have in the file
    //loc1 = 0; 
  //}
  //else{    
    firstInCachTime = MainHeader::fullTSJustBefore(fileSize - Cache::cacheSize_);
    firstInCachTime = (firstInCachTime & 0b11111111111111110000000000000000) 
                      | Cache::getFirstLowTime();

    //check if the wanted timestamp could be in the cache
    //std::cout<<"Tstamp1: "<<Tstamp1<<" Data::cacheOldestT_: "<<firstInCachTime<<"\n";
    //if (Tstamp1 > firstInCachTime){TODO implement
    if (false){//TODO placeholder
      loc1 = findTimestamp_inCache(Tstamp1, startSearch, stopSearch, fileSize);
    }
    else{
      uint16_t time1Low = (uint16_t)(Tstamp1 & 0b00000000000000001111111111111111);
      loc1 = findTimestamp_inFile_lowerBound(time1Low, startSearch, stopSearch);
      db("loc1: "<<loc1<<"\n")
    }
  //}

  MainHeader::findFullTS(Tstamp2, startSearch, stopSearch);
  if(stopSearch == -1){stopSearch = fileSize; } //handle case Tstamp2 > last full TS   
    if (false){
      loc2 = findTimestamp_inCache(Tstamp2, startSearch, stopSearch, fileSize);
    }
    else{
      loc2 = findTimestamp_inFile_upperBound(Tstamp2, startSearch, stopSearch);
    }
  //}
  std::cout<<"loc1: "<<loc1<<"\tloc2: "<<loc2<<"\n";
}

int Data::findTimestamp_inFile_lowerBound(uint16_t TS_low, unsigned int startSearch, unsigned int stopSearch){
  
  std::cout<<"enterd lowerbound\n";
  std::cout<<"startSearch: "<<startSearch<<", stopSearch: "<<stopSearch<<"\n";
  uint16_t timelow;

  unsigned int nBlocks;
  unsigned int blockSize_B;

  unsigned int rest_B;
  unsigned int orgIdx_B;

  uint8_t block[MAXBLOCKSIZE];

  //calculate how many blocks we need
  //if(stopByte-startByte>MAXBLOCKSIZE){nBlocks = (stopByte - startByte)/MAXBLOCKSIZE; } 
  //else{nBlocks = 1;}
  //FIXME TEMP REMOVED IF ELSE DONT SEE ITS USE
  nBlocks = (stopSearch - startSearch)/MAXBLOCKSIZE;
  //determine blocksize in bytes
  blockSize_B = std::min(MAXBLOCKSIZE - (MAXBLOCKSIZE%packageSize_), stopSearch-startSearch); 
  rest_B = (stopSearch-startSearch)%MAXBLOCKSIZE; //number of bytes that doesnt fit in the normal blocks
  
  //FIXME DEBUG
  //timeHigh = MainHeader::fullTSJustBefore(4) & 0b11111111111111110000000000000000;
  //std::cout<<"timestamp we want: "<< ((uint32_t)TS_low | timeHigh)<<"\n";
  //std::cout<<"timeHigh used: "<< +timeHigh<<"\n";
  //FIXME DEBUG
  
  
  //iterate over the blocks
  for (unsigned int i = 0; i < nBlocks; i++) {
    //read one block to memory
    fseek(fileP_, startSearch+i*blockSize_B, SEEK_SET);
    fread(block, 1, blockSize_B, fileP_);

    //iterate through the block in memory 
    for (unsigned int blockIdx_B = 0; blockIdx_B < blockSize_B; blockIdx_B+=packageSize_) {
      timelow = (uint16_t)block[blockIdx_B+1] << 8 |
                (uint16_t)block[blockIdx_B];
      if(timelow >= TS_low){
        orgIdx_B = startSearch+i*blockSize_B+ blockIdx_B;  
        //std::cout<<"fulltime here is: "<< +((uint32_t) timelow | timeHigh) <<"\n";   
        //std::cout<<"HERE1";
        if(timelow == TS_low){return orgIdx_B;}
        else{return orgIdx_B-packageSize_;} //to force inclusion of first time
      }
    }
  }

  //do the leftover values in a smaller block
  fseek(fileP_, stopSearch-rest_B, SEEK_SET);
  fread(block, 1, rest_B, fileP_);

  //iterate through the block in memory in bin groups
  for (unsigned int blockIdx_B = 0; blockIdx_B < rest_B; blockIdx_B+=packageSize_) {
    timelow = (uint16_t)block[blockIdx_B+1] << 8 |
              (uint16_t)block[blockIdx_B];
    if(timelow >= TS_low){
      int orgIdx_B = startSearch+nBlocks*blockSize_B+ blockIdx_B;     
      //std::cout<<"HERE2\n";
      //std::cout<<startSearch<<"\n"<<nBlocks<<"\n"<<blockSize_B<<"\n"<<blockIdx_B<<"\n";
      if(timelow == TS_low){return orgIdx_B;}
      else{return orgIdx_B-packageSize_;} //to force inclusion of first time
    }
  }
  //every value in the range is smaller then the wanted timestamp
  //std::cout<<"HERE3";
  return stopSearch; //range is thus the best approximation.
}

int Data::findTimestamp_inFile_upperBound(uint32_t TS, unsigned int startSearch, unsigned int stopSearch){

  std::cout<<"enterd upperbound\n";
  std::cout<<"startSearch: "<<startSearch<<", stopSearch: "<<stopSearch<<"\n";

  uint16_t timelow;
	uint16_t TS_low;

  unsigned int nBlocks;
  unsigned int blockSize_B;

  unsigned int rest_B;
  unsigned int orgIdx_B;

  uint8_t block[MAXBLOCKSIZE];

  //calculate how many blocks we need
  //if(stopByte-startByte>MAXBLOCKSIZE){nBlocks = (stopByte - startByte)/MAXBLOCKSIZE; } 
  //else{nBlocks = 1;}
  //FIXME TEMP REMOVED IF ELSE DONT SEE ITS USE
  nBlocks = (stopSearch - startSearch)/MAXBLOCKSIZE;
  //determine blocksize in bytes
  blockSize_B = std::min(MAXBLOCKSIZE - (MAXBLOCKSIZE%packageSize_), stopSearch-startSearch); 
  rest_B = (stopSearch-startSearch)%MAXBLOCKSIZE; //number of bytes that doesnt fit in the blocks
  
  //FIXME DEBUG
  
  //std::cout<<"timestamp we want: "<< ((uint32_t)TS_low | timeHigh)<<"\n";
  //std::cout<<"timeHigh used: "<< +timeHigh<<"\n";
  //FIXME DEBUG
  
  //std::cout<<"HEREERO\n";
  //std::cout<<nBlocks<<"\n";
  //std::cout<<blockSize_B<<"\n";
  //std::cout<<rest_B<<"\n";
//  std::cout<<blockIdx_B<<"\n";
  
  //do the leftover values in a smaller block
  fseek(fileP_, stopSearch-rest_B, SEEK_SET);
  fread(block, 1, rest_B, fileP_);

	/*check if the wanted value even lies within the search range*/
	timelow = (uint16_t)block[rest_B-packageSize_+1] << 8 |
            (uint16_t)block[rest_B-packageSize_];	

	timeHigh = MainHeader::fullTSJustBefore(stopSearch) & 0b11111111111111110000000000000000; 
	uint32_t fulltime = (uint32_t) timelow | timeHigh;
	if(fulltime < TS){return stopSearch;}
	TS_low = (uint16_t)TS;

  //iterate through the block in memory in bin groups
  for (int blockIdx_B = rest_B-packageSize_; blockIdx_B >= 0; blockIdx_B-=packageSize_) {
    timelow = (uint16_t)block[blockIdx_B+1] << 8 |
              (uint16_t)block[blockIdx_B];
    //std::cerr<<blockIdx_B<<" ";
    if(timelow <= TS_low){
      orgIdx_B = stopSearch-rest_B + blockIdx_B;
      //std::cout<<"fulltime here is: "<< +((uint32_t) timelow | timeHigh) <<"\n";
      //std::cout<<"HEREERO1\n";
      //std::cout<<"orgIdx_B"<<orgIdx_B<<"\n";
      return orgIdx_B+packageSize_;
    }
  }
  //std::cerr<<"next\n";
  
  //iterate over the blocks
  for (int i = nBlocks-1; i >= 0; i--) {
    //read one block to memory
    fseek(fileP_, startSearch+i*blockSize_B, SEEK_SET);
    fread(block, 1, blockSize_B, fileP_);

    //iterate through the block in memory 
    for (int blockIdx_B = blockSize_B-packageSize_; blockIdx_B >= 0; 
         blockIdx_B-=packageSize_) {
      timelow = (uint16_t)block[blockIdx_B+1] << 8 |
                (uint16_t)block[blockIdx_B];
      //std::cerr<<blockIdx_B<<" ";
      if(timelow <= TS_low){
        int orgIdx_B = startSearch+i*blockSize_B+ blockIdx_B;     
        //std::cout<<"fulltime here is: "<< +((uint32_t) timelow | timeHigh) <<"\n";
        //std::cout<<"HEREERO2\n";
        //std::cout<<i<<", "<<blockSize_B<<", "<<blockIdx_B<<"\n";
        //std::cout<<"orgIdx_B"<<orgIdx_B<<"\n";
        return orgIdx_B+packageSize_;
      }
    }
  }

  //every value in the range is larger then the wanted timestamp the start of the
  return startSearch; //range is thus the best approximation.
}//done













//DATAFETCH HELP FUNCT
Data::iterator::iterator(unsigned int startByte, unsigned int stopByte, unsigned int packageSize){//TODO implement ignoring extra datapoints
  unsigned int numbUnusable; //numb of values we can use for plotting without
  //going over MAXPLOTRESOLUTION.
  unsigned int numbOfValues;
  
  numbOfValues = (stopByte-startByte)/packageSize;
  numbUnusable = numbOfValues%MAXPLOTRESOLUTION; 

  if(numbOfValues < MAXPLOTRESOLUTION){binSize_P = 1; }
  else{binSize_P = numbOfValues/MAXPLOTRESOLUTION;}
  
  if(numbOfValues < MAXPLOTRESOLUTION || numbUnusable == 0){spacing = -1; }
  else{spacing = (numbOfValues-1)/numbUnusable; }//-1 to compensate for counter starting at 1
  counter = 1;//counter starting at 1 to prevent i=0 to evaluate as always false
}

bool Data::iterator::useValue(unsigned int i){
  //calculate if element 'i' should be used or not
  if(i == (unsigned int)(counter*spacing)){
    counter++;
    //std::cout<<"ignored "<<counter-1<<" datapoints"<<"\t\tindex was: "<<i<<"\n";
    return false;
  }
  else{return true;}
}

void Data::initGetTime(int startByte){
  timeHigh = MainHeader::fullTSJustBefore(startByte) & 0b11111111111111110000000000000000;
  prevTimePart[0] = 0;
  prevTimePart[1] = 0;
  std::cerr<<"timeHigh :"<<timeHigh<<"startByte: "<<startByte<<"\n";
}
 
uint32_t Data::getTime(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]){
  uint16_t timelow;
  uint32_t fullTimeStamp;
  
  timelow = (uint16_t)block[blockIdx_B+1] << 8  |
            (uint16_t)block[blockIdx_B];
  //std::cout<<"timelow: "<<timelow<<"\ttimeHigh: "<<(timeHigh)<<"\n";
  fullTimeStamp = timeHigh | (uint32_t)timelow;
  db("fullTimeStamp: "<<fullTimeStamp<<" \n")
  return fullTimeStamp;
}

double Data::meanT(uint32_t* array, int len){
  uint32_t Mean = 0;
  uint32_t first = *(array+0);
  for(int i = 1; i<len; i++){ Mean = Mean+*(array+i)-first;}
  Mean /= len;
  Mean += first;
  return (double)Mean;
}

uint16_t Data::meanB(uint16_t* array, int len){
  uint16_t Mean = 0;
  for(int i =0; i<len; i++){ Mean = Mean | *(array+i);}
  return Mean;
}

//HELPER FUNCT
void Data::putFullTS(const uint32_t Tstamp){
  int currentByte;
  prevFTstamp = Tstamp;
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
