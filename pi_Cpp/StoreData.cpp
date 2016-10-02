#include "StoreData.h"
#include <chrono>

StoreData::StoreData(){
	
	mkdir("data", S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
  sensDatFile = fopen("data/enviremental.binDat", "a");
  pirDatFile = fopen("data/pirs.binDat", "a");
  
  prevPirData[1] = 0;//0 as in no pirs measured
  lastPirBegin = std::chrono::high_resolution_clock::now();    
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

void StoreData::pir_write(unsigned char data[2]){
  const static char DATASIZE = 2;
  fwrite(data, DATASIZE, DATASIZE*sizeof(unsigned char), pirDatFile);
	//TODO add time
	
	std::cout << "wrote some shit \n";
}

void StoreData::pir_convertNotation(unsigned char& B[2]){
  unsigned char B_ones, B_zeros;

  B_ones  = B[0] & B[1]; //if one and noted as correct (one) store as one
  B_zeros = (B[0] ^ B[1]) & B[1]; //if zero and noted as correct: if (zero and one) only if also one

  B = F_ones | F_zeros; //back to old notation [one or zero][correct or not]  
}

void StoreData::pir_combine(unsigned char& B[2]){
  unsigned char A_ones, A_zeros, B_ones, B_zeros;
  unsigned char F_ones, F_zeros; //zeros is 1 if a zero was confirmed at that place
  
  //First convert to the new notation
  pir_convertNotation(prevPirData);
  
  A_ones  = prevPirData[0];
  A_zeros = prevPirData[1];

  B_ones = B[0];
  B_zeros = B[1];
  
  F_ones = (A_ones & ~ B_zeros) | B_ones; //if was one and not zero now or if one now = one
  F_zeros = (A_zeros & ~ B_ones) | B_zeros; //if was zero and not one now or if zero now = one
                                            //also one here as one indicates a correct zero in B_zeros
  
  B = F_ones | F_zeros; //back to old notation [one or zero][correct or not]
}

void StoreData::pir_binData(unsigned char data[2]){
  std::chrono::time_point timepassed;
  timepassed = chrono::high_resolution_clock::now() - lastPirBegin;
  timepassed = std::chrono::duration_cast<std::chrono::microseconds>(timepassed).count()
  if (timepassed < PIR_DT){
    //add movement values to pir
    pirRecord = pirRecord[0] | data[0]; 
    pirRecord = pirRecord[0] | data[0];
  }
  else{
    //write values collected till now
    pir_write(pirRecord);  
    std::chrono::time_point begin = std::chrono::high_resolution_clock::now();    
    
    //reset pir to new values
    pirRecord[0] = 0;
    pirRecord[1] = 0;
  }
}

bool StoreData::pir_checkIfSame(unsigned char data[2]){
  if ((data[0] == prevPirData[0]) & (data[1] = prevPirData[1])){ return true;}
  else{return false;}
}

void StoreData::pir_process(unsigned char data[2]){
  unsigned char combinedCorrect;
  unsigned char newCorrect;
  

  if (!pir_checkIfSame(data)){
      
      combinedCorrect = prevPirData[1] & data[1]; 
      newCorrect = data[1];
      
      pir_convertNotation(data);      
      if (combinedCorrect > newCorrect){ //would comparing with prev data increase knowledge?
        pir_combine(data); //combine data with newer data overriding older data
      }
      prevPirData = data;
      pir_binData(data); //bin on time and write when neccesairy    
    }
  }
}
//
