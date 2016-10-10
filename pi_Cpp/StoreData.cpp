#include "StoreData.h"

StoreData::StoreData(){
	
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
	//open file in binairy mode with appending write operations
  sensDatFile = fopen("data/enviremental.binDat", "a+b");
  pirDatFile = fopen("data/pirs.binDat", "a+b");  
  
  prevPirData[1] = 0;//0 as in no pirs measured
  t_begin = GetMilliSec();
  pirRecord[0] = 0;
  pirRecord[1] = 0;
}

StoreData::~StoreData(){
  fclose(sensDatFile);
  fclose(pirDatFile);
}

bool StoreData::pir_isTimeStampPackage(unsigned char susp_time[4],  unsigned char susp_data[4]){
  if (susp_time[2] != susp_data[0]){
    if (susp_time[3] != susp_data[1]){ return true; }    
  }
  return false;
}

void StoreData::pir_readLine(int lineNumber){
  unsigned char askedLine[4];
  uint32_t unix_time;
  unsigned char susp_time[4];
  unsigned char susp_data[4];
  int n = 0;

  fseek(pirDatFile, 4*(lineNumber-n), SEEK_SET); 
  fread(askedLine, 1, 4, pirDatFile);

  //TODO needs to check if first package is a time package

  std::memcpy(susp_time, askedLine, 4);
  do {
    std::memcpy(susp_data, susp_time, 4);
    //read packages in front of sups_data, store in susp_time
    n++;
    fseek(pirDatFile, 4*(lineNumber-n), SEEK_SET);    
    fread(susp_time, 1, 4, pirDatFile);   
  } while (!pir_isTimeStampPackage(susp_time, susp_data));
  
  //conversion back to full unix time  
  unix_time = (uint32_t)susp_time[1] << 24 | //shift high part 
              (uint32_t)susp_time[0] << 16 | //
              (uint32_t)susp_time[3] << 8  | //shift and add low part
              (uint32_t)susp_time[2];
  
  std::cout << "Reading timestamp: "
		 			<< +susp_time[0] << " "
					<< +susp_time[1] << " "
			 		<< +susp_time[2] << " "
			 		<< +susp_time[3] << "\n";
  
  std::cout << unix_time << "\n";
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

  unsigned char towrite[4];
  
  //conversion to bytes
  towrite[2] = timestamp & 0xff;         //store low part in last two bytes
  towrite[3] = (timestamp >> 8) & 0xff;  
  towrite[0] = (timestamp >> 16) & 0xff; //store high part in first 2 bytes
  towrite[1] = (timestamp >> 24) & 0xff;  
  
  fwrite(towrite, 4, 4*sizeof(unsigned char), pirDatFile);
  std::cout << "writing TIMESTAMP PIR PACKAGE: "
  		 			<< +towrite[0] << " "
						<< +towrite[1] << " "
				 		<< +towrite[2] << " "
				 		<< +towrite[3] << "\n";
}	
	
void StoreData::pir_write(unsigned char data[2]){	
  long int timestamp;				
  unsigned char buffer[4];
  
	timestamp = unix_timestamp();	
  uint16_t timeLow = (uint16_t) (timestamp >> 16);// shifting right 16 times gives /2^16
	
	if(!TimeStampSet_first & (timeLow <= HALFDAYSEC)){
	  //second condition is needed to get a timestamp after a restart
	  pir_writeTimestamp(timestamp);
	  TimeStampSet_first = true;
	  TimeStampSet_second = false;
	}
	else if(!TimeStampSet_second & (timeLow > HALFDAYSEC)){
	  pir_writeTimestamp(timestamp); 
	  TimeStampSet_second = true;
	  TimeStampSet_first = false;
	}

  //store timeLow in buffer
  buffer[0] = timeLow & 0xff;
  buffer[1]	= (timeLow >> 8) & 0xff;

	std::memcpy(buffer+2, data, 2);	
	fwrite(buffer, 4, 4*sizeof(unsigned char), pirDatFile);	
  std::cout << "writing NORMAL PIR PACKAGE: "
  		 			<< +buffer[0] << " "
						<< +buffer[1] << " "
				 		<< +buffer[2] << " "
				 		<< +buffer[3] << "\n";

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
