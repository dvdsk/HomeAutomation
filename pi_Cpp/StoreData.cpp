#include "StoreData.h"

StoreData::StoreData(){
	
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  sensDatFile = fopen("data/enviremental.binDat", "a");
  pirDatFile = fopen("data/pirs.binDat", "a");
  
  prevPirData[1] = 0;//0 as in no pirs measured
  t_begin = GetMilliSec();
  pirRecord[0] = 0;
  pirRecord[1] = 0;
}

StoreData::~StoreData(){
  fclose(sensDatFile);
  fclose(pirDatFile);
}


void StoreData::envirmental_write(unsigned char data[18]){
  const static char DATASIZE = 18;
  fwrite(data, DATASIZE, DATASIZE*sizeof(unsigned char), sensDatFile);
	//TODO add time
	
	std::cout << "wrote some shit \n";
}
	
long int StoreData::unix_timestamp() {
  time_t t = std::time(0);
  long int now = static_cast<long int> (t);
  return now;
}

long long StoreData::GetMilliSec(){
  struct timeval tp;
  gettimeofday(&tp, NULL);
  //get current timestamp in milliseconds
  long long mslong = (long long) tp.tv_sec * 1000 + tp.tv_usec / 1000; 
  return mslong;
}


void StoreData::pir_writeTimestamp(long int timestamp){

  union {
  long int      i;
  unsigned char bytes[4];
  } longInt_bytes;
  
  fwrite(longInt_bytes.bytes, 4, 4*sizeof(unsigned char), pirDatFile);
}	
	
void StoreData::pir_write(unsigned char data[2]){	
  long int timestamp;				
  unsigned short partOfHalfDay;
  unsigned char buffer[4];
  
  union {
    int           i;
    unsigned char bytes[2];
  } int_bytes;
  
	timestamp = unix_timestamp();
	partOfHalfDay = timestamp % HALFDAYSEC;	
	
	if(!TimeStampSet_first){
	  pir_writeTimestamp(timestamp);
	  TimeStampSet_first = true;
	}
	else if(!TimeStampSet_second & (partOfHalfDay > HALFDAYSEC/2)){
	  pir_writeTimestamp(timestamp); 
	  TimeStampSet_second = true;
	}
	
	int_bytes.i = partOfHalfDay;
	std::memcpy(buffer, int_bytes.bytes, 2);
	std::memcpy(buffer, data, 2);	
	fwrite(buffer, 4, 4*sizeof(unsigned char), pirDatFile);	
	
	TimeStampSet_first = false;
	TimeStampSet_second = false;
}


void StoreData::pir_convertNotation(unsigned char B[2]){
  unsigned char B_ones, B_zeros;

  B_ones  =  B[0] & B[1]; //if one and noted as correct (one) store as one
  B_zeros = (B[0] ^ B[1]) & B[1]; //if zero and noted as correct: if (zero and one) only if also one

  B[0] = B_ones;
  B[1] = B_zeros; //back to old notation [one or zero][correct or not]  
}

void StoreData::pir_combine(unsigned char B[2]){
  short int A_ones, A_zeros, B_ones, B_zeros;
  short int F_ones, F_zeros; //zeros is 1 if a zero was confirmed at that place

  unsigned char prevPirData_new[2];  
  
  //First previous runs data to new notation (kept in old for same check)
  std::memcpy(prevPirData_new, prevPirData,2);
  pir_convertNotation(prevPirData_new);
  
  A_ones  = prevPirData[0];
  A_zeros = prevPirData[1];

  B_ones = B[0];
  B_zeros = B[1];
  
  F_ones = (A_ones & ~ B_zeros) | B_ones; //if was one and not zero now or if one now = one
  F_zeros = (A_zeros & ~ B_ones) | B_zeros; //if was zero and not one now or if zero now = one
                                            //also one here as one indicates a correct zero in B_zeros
  
  B[0] = F_ones;
  B[1] = F_zeros;
}

void StoreData::pir_binData(unsigned char data[2]){
  long long timepassed;
  timepassed = GetMilliSec() - t_begin;

  if (timepassed < PIR_DT){
    //add movement values to pir
    pirRecord[0] = pirRecord[0] | data[0]; 
    pirRecord[1] = pirRecord[1] | data[1];
  }
  else{
    //write values collected till now
    pir_write(pirRecord);  
    t_begin = GetMilliSec();
    
    //reset pir to new values
    pirRecord[0] = 0;
    pirRecord[1] = 0;
  }
}

bool StoreData::pir_isNotSame(unsigned char data[2]){
  if ((data[0] == prevPirData[0]) & (data[1] == prevPirData[1])){ return false;}
  else{return true;}
}

void StoreData::pir_process(unsigned char data[2]){
  unsigned char combinedCorrect;
  unsigned char newCorrect;
  

  if (pir_isNotSame(data)){
      
    combinedCorrect = prevPirData[1] & data[1]; 
    newCorrect = data[1];
    std::memcpy(prevPirData, data, 2); //save before notation is changed

    pir_convertNotation(data);      
    if (combinedCorrect > newCorrect){ //would comparing with prev data increase knowledge?
      pir_combine(data); //combine data with newer data overriding older data
    }      
    pir_binData(data); //bin on time and write when neccesairy    
  }
}

//
