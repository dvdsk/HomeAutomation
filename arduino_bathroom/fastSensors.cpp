#include "fastSensors.h"

void readAndEncode(uint8_t buffer[]){
	buffer[0] |= readPIRs();
}

uint8_t readPIRs(){
	uint8_t pirStat;
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second

	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	delay(1);//crashes if removed  TODO checkthis!!!
	pirStat =  PIND & PIR_SHOWER >> 2; 
	pirStat |= PIND & PIR_WC >> 2;

	return pirStat;  //set bedSouth value to recieved data
}

void configure_fast(){
	DDRD &= !PIR_SHOWER;
	DDRD &= !PIR_WC;

}
