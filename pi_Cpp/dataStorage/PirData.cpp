	
void PirData::putData(unsigned char data[2]){	
  long int timestamp;				
  unsigned char buffer[4];
  
	timestamp = unix_timestamp();	
  uint16_t timeLow = (uint16_t) (timestamp >> 16);// shifting right 16 times gives /2^16
	
	if(!TimeStampSet_first & (timeLow <= HALFDAYSEC)){
	  //second condition is needed to get a timestamp after a restart
	  putTimestamp(timestamp);
	  TimeStampSet_first = true;
	  TimeStampSet_second = false;
	}
	else if(!TimeStampSet_second & (timeLow > HALFDAYSEC)){
	  putTimestamp(timestamp); 
	  TimeStampSet_second = true;
	  TimeStampSet_first = false;
	}

  //store timeLow in buffer
  buffer[0] = timeLow & 0xff;
  buffer[1]	= (timeLow >> 8) & 0xff;

	std::memcpy(buffer+2, data, 2);	
	//TODO call write funct
	
	dataStorage.write_pir(buffer);
  std::cout << "writing NORMAL PIR PACKAGE: "
  		 			<< +buffer[0] << " "
						<< +buffer[1] << " "
				 		<< +buffer[2] << " "
				 		<< +buffer[3] << "\n";

}


void PirData::convertNotation(unsigned char B[2]){
  unsigned char B_ones, B_zeros;

  B_ones  =  B[0] & B[1]; //if one and noted as correct (one) store as one
  B_zeros = (B[0] ^ B[1]) & B[1]; //if zero and noted as correct: if (zero and one) only if also one

  B[0] = B_ones;
  B[1] = B_zeros; //back to old notation [one or zero][correct or not]  
}

void PirData::combine(unsigned char B[2]){
  short int A_ones, A_zeros, B_ones, B_zeros;
  short int F_ones, F_zeros; //zeros is 1 if a zero was confirmed at that place

  unsigned char prevData_new[2];  
  
  //First previous runs data to new notation (kept in old for same check)
  std::memcpy(prevData_new, prevData,2);
  convertNotation(prevData_new);
  
  A_ones  = prevData[0];
  A_zeros = prevData[1];

  B_ones = B[0];
  B_zeros = B[1];
  
  F_ones = (A_ones & ~ B_zeros) | B_ones; //if was one and not zero now or if one now = one
  F_zeros = (A_zeros & ~ B_ones) | B_zeros; //if was zero and not one now or if zero now = one
                                            //also one here as one indicates a correct zero in B_zeros
  
  B[0] = F_ones;
  B[1] = F_zeros;
}

void PirData::binData(unsigned char data[2]){
  long timepassed;
  timepassed = unix_timestamp() - t_begin;

  if (timepassed < PIR_DT){
    //add movementDetect values to pir
    Record[0] = Record[0] | data[0]; 
//    Record[1] = Record[1] | data[1];
  }
  else{
    //write values collected till now
    putData(Record);  
    t_begin = unix_timestamp();
    
    //reset pir to new values
    Record[0] = 0;
    Record[1] = 0;
  }
}

bool PirData::isNotSame(unsigned char data[2]){
  if ((data[0] == prevData[0]) & (data[1] == prevData[1])){ return false;}
  else{return true;}
}

void PirData::process(unsigned char data[2]){
  unsigned char combinedCorrect;
  unsigned char newCorrect;
  

  if (isNotSame(data)){//als de data niet hetzelfde is
      
    combinedCorrect = prevData[1] & data[1]; //number of 'correctly' read sensors
    newCorrect = data[1]; //new correct data
    std::memcpy(prevData, data, 2); //save current data into prevData before we
    //switch notations to the storage format.

    convertNotation(data);      
    if (combinedCorrect > newCorrect){ //would merging with prev data increase knowledge?
      combine(data); //combine old data with newer data overriding older data
    }      
    binData(data); //bin on time and write when neccesairy    
  }
}

//
