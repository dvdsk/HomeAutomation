#include "fastSensors.h"

BH1750 lightSens1(false, BH1750_CONTINUOUS_HIGH_RES_MODE);
BH1750 lightSens2(true, BH1750_CONTINUOUS_HIGH_RES_MODE);

void begin(){
	DDRD &= !PIR_SHOES_WEST;
	DDRD &= !PIR_SHOES_EAST;
	DDRD &= !PIR_DOOR;
	DDRD &= !PIR_KITCHEN;

	lightSens1.begin();
	lightSens2.begin();
}

void readAndEncode(uint8_t buffer[]){
	db("in readAndEncode") 
	buffer[0] |= readPIRs();

	encode(buffer, lightSens1.readLightLevel(), EncFastArduino::LIGHT_DOOR, 
	       EncFastArduino::LEN_LIGHT);
	encode(buffer, lightSens2.readLightLevel(), EncFastArduino::LIGHT_KITCHEN, 
	       EncFastArduino::LEN_LIGHT);
	db("out readAndEncode") 
}

uint8_t readPIRs(){
	db("in readPIRs") 
	uint8_t pirStat;
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second

	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	pirStat =  PIND & PIR_SHOES_WEST >> 2; //TODO FIXME check the shift number
	pirStat |= PIND & PIR_SHOES_EAST >> 2;
	pirStat |= PIND & PIR_DOOR >> 2; 
	pirStat |= PIND & PIR_KITCHEN >> 2;

	db("out readPIRs") 
	return pirStat;  //set bedSouth value to recieved data
}

void configure_fast(){
	db("in configure_fast") 
	pinMode(PIR_SHOES_WEST, INPUT);
	pinMode(PIR_SHOES_EAST, INPUT);
	pinMode(PIR_DOOR, INPUT);
	pinMode(PIR_KITCHEN, INPUT);

	lightSens1.begin();
	lightSens2.begin();
}
