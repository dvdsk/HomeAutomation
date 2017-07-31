#include "fastSensors.h"

void readAndEncode(uint8_t buffer[]){
	buffer[0] |= readPIRs();
	encode(buffer, readLight(), EncFastArduino::LIGHT_BED, 
	       EncFastArduino::LEN_LIGHT)
}

uint8_t readPIRs(){
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second

	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	delay(1);//crashes if removed  TODO checkthis!!!
	return PINA & (PIR_SOUTH | PIR_NORTH);  //set bedSouth value to recieved data
}

uint16_t readLight(){
	//read light sensor (anolog) and return over serial, this happens multiple times
	//a second, convert the data to binairy and send using hte following format:
	//[header for this light sensor (see the top of file)][lightLevel byte 1]
	//[light level byte 2]
	
	return analogRead(pin::LIGHT_BED);    // read the input pin
}

