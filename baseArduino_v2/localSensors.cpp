#include "localSensors.h"

void LocalSensors::setup(uint16_t* fastData_){fastData = fastData_; }

void LocalSensors::updateFast_Local(){
	readPIRs();
	readLight();
}

void LocalSensors::readPIRs(){
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second
	
	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	delay(1);//crashes if removed  TODO checkthis!!!
	*(fastData+0) = PINL | 0b10000000;  //set bedSouth value to recieved data
	*(fastData+1) = 0b10000000;  //location on pinbank
}

void LocalSensors::readLight(){
	//read light sensor (anolog) and return over serial, this happens multiple times
	//a second, convert the data to binairy and send using hte following format:
	//[header for this light sensor (see the top of file)][lightLevel byte 1]
	//[light level byte 2]
	
	INTUNION_t light;
	
	light.number = analogRead(pin::LIGHT_BED);    // read the input pin
	*(fastData+5) = light.number;
}
