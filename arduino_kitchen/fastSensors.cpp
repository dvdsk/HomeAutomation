#include "fastSensors.h"

FastSensors::FastSensors() : lightSens1(true), lightSens2(false)
{

}

void FastSensors::begin(){
	DDRD &= !PIR_SHOES_WEST;
	DDRD &= !PIR_SHOES_EAST;
	DDRD &= !PIR_DOOR;
	DDRD &= !PIR_KITCHEN;

	lightSens1.begin();
	lightSens2.begin();
}

void FastSensors::readAndEncode(uint8_t buffer[]){
	buffer[0] |= readPIRs();

	Serial.println("reading door");
	encode(buffer, lightSens1.readLightLevel(), EncFastArduino::LIGHT_DOOR, 
	       EncFastArduino::LEN_LIGHT);
	Serial.println("reading kitchen");
	encode(buffer, lightSens2.readLightLevel(), EncFastArduino::LIGHT_KITCHEN, 
	       EncFastArduino::LEN_LIGHT);
	Serial.println("done");
}

uint8_t FastSensors::readPIRs(){
	uint8_t pirStat;
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second

	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	delay(1);//crashes if removed  TODO checkthis!!!
	pirStat =  PIND & PIR_SHOES_WEST >> 2; 
	pirStat |= PIND & PIR_SHOES_EAST >> 2;
	pirStat |= PIND & PIR_DOOR >> 2; 
	pirStat |= PIND & PIR_KITCHEN >> 2;

	return pirStat;  //set bedSouth value to recieved data
}
