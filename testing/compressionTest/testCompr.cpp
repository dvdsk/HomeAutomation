#include "compression.h"
#include "config.h"
#include "encodingScheme.h"

#include <iostream>

char commandBuffer[3];
uint8_t commandBuffer_Len = 0;

//first element of slowdata used to check which values have been updated
uint16_t slowData[SLOWDATA_SIZE];
uint16_t fastData[FASTDATA_SIZE];

////////////////////////////////////////////////////
////////////////////////////////////////////////////


int main(){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled

	uint8_t toSend[Enc_slow::LEN_ENCODED+1];
	uint16_t decoded;
  
	for(int i=0; i< 1600; i++){
		memset(toSend, 0, Enc_slow::LEN_ENCODED+1);
		slowData[Idx::UPDATED] = 0;  

		slowData[Idx::CO2] = i;
		encode(toSend, slowData[Idx::CO2],
			Enc_slow::CO2, Enc_slow::LEN_CO2);

		slowData[Idx::PRESSURE] = 1200; //max value before things go wrong (7 bits)
		encode(toSend, slowData[Idx::PRESSURE],
			Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);
	
		if(decode(toSend, Enc_slow::CO2, Enc_slow::LEN_CO2) != i){
			std::cout<<"ERROR at i="<<i<<"\n";
			
		}
}

//	
//	//decoded = decode(toSend, Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);
//	decoded = decode(toSend, Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);
//	std::cout<<"pressure2: ";
//	std::cout<<decoded;	
//	std::cout<<", pressure-org2: ";
//	std::cout<<slowData[Idx::PRESSURE];

//	decoded = decode(toSend, Enc_slow::CO2, Enc_slow::LEN_CO2);
//	std::cout<<"\nCO2: ";
//	std::cout<<decoded;
//	std::cout<<", CO2-org: ";
//	std::cout<<slowData[Idx::CO2];

//	std::cout<<"\nLEN_ENCODED: ";
//	std::cout<<Enc_slow::LEN_ENCODED;

//	std::cout<<", PRESSURE: ";
//	std::cout<<Enc_slow::PRESSURE;
//	std::cout<<"\n";

}

//int main(){

//	for(int compLen =0; compLen
//	
//}
