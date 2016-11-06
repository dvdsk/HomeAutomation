
PirData::PirData()
: Data(filePath, cache, packageSize, cacheLen){

}

void PirData::process(uint8_t rawData[2], uint32_t Tstamp){
  
  if (newData(rawData) ){
    convertNotation(rawData);

  }

}



bool PirData::newData(uint8_t raw[2]){
  if ((raw[0] == prevRaw[0]) & (raw[1] == prevRaw[1])){ return false;}
  else{
    std::memcpy(prevData, data , 2);
    return true;
  }
}

void PirData::convertNotation(uint8_t rawData[2]){
  uint8_t confirmed_one, confirmed_zero;

  uint8_t oneOrZero = rawData[0];
  uint8_t confirmed = rawData[1];

  confirmed_one  = oneOrZero & confirmed; //if one and noted as correct (one) give one
  confirmed_zero = (oneOrZero ^ confirmed) & confirmed;
  
  /*explanation of confirmed zero algoritme
    we want only OnOff=F and confirmed=T to give T. the ^ (XOR) operator has 
    the following possible outcomes:
      OnOff:      F F T T
      confirmed:  F T F T
      outcome:    F T T F
    XOR gives us half of what we want. To filter out the confirmed=F case we
    do an AND operation. Now we have T where a zero is confirmed*/
}

////////////OLD///////////////

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
