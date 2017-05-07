#include "compression.h"
#include "config.h"
#include "encodingScheme.h"

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
  
	uint8_t toSend[Enc_slow::LEN_ENCODED];
	memset(toSend, 0, Enc_slow::LEN_ENCODED);
	slowData[Idx::UPDATED] = 0;  
	
	encode(toSend, slowData[Idx::TEMPERATURE_BED], 
		Enc_slow::TEMP_BED, Enc_slow::LEN_TEMP);
	encode(toSend, slowData[Idx::TEMPERATURE_DOOR], 
		Enc_slow::TEMP_DOOR, Enc_slow::LEN_TEMP);
	encode(toSend, slowData[Idx::TEMPERATURE_BATHROOM], 
		Enc_slow::TEMP_BATHROOM, Enc_slow::LEN_TEMP);

	encode(toSend, slowData[Idx::HUMIDITY_BED],
		Enc_slow::HUM_BED, Enc_slow::LEN_HUM);
	encode(toSend, slowData[Idx::HUMIDITY_DOOR], 	 			
		Enc_slow::HUM_DOOR, Enc_slow::LEN_HUM);	
	encode(toSend, slowData[Idx::HUMIDITY_BATHROOM],		
		Enc_slow::HUM_BATHROOM, Enc_slow::LEN_HUM);

	encode(toSend, slowData[Idx::CO2],
		Enc_slow::CO2, Enc_slow::LEN_CO2);

	slowData[Idx::PRESSURE] = 127; //max value before things go wrong (7 bits)

	encode(toSend, slowData[Idx::PRESSURE],
		Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);

/*	encode(toSend, 11,*/
/*		Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);*/

	std::cout<<"pressure: ";
	std::cout<<decode(toSend, Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);
	std::cout<<", pressure-org: ";
	std::cout<<slowData[Idx::PRESSURE];

	slowData[Idx::PRESSURE] = 128; //max value before things go wrong (7 bits)

	encode(toSend, slowData[Idx::PRESSURE],
		Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);

	std::cout<<"\npressure2: ";
	std::cout<<decode(toSend, Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);
	std::cout<<", pressure-org2: ";
	std::cout<<slowData[Idx::PRESSURE];

	std::cout<<"\nCO2: ";
	std::cout<<decode(toSend, Enc_slow::CO2, Enc_slow::LEN_CO2);
	std::cout<<", CO2-org: ";
	std::cout<<slowData[Idx::CO2];

	std::cout<<"\nLEN_ENCODED: ";
	std::cout<<Enc_slow::LEN_ENCODED;

	std::cout<<", PRESSURE: ";
	std::cout<<Enc_slow::PRESSURE;
	std::cout<<"\n";

}

