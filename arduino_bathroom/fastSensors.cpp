#include "fastSensors.h"

void readAndEncode(uint8_t buffer[]){
	db("in readAndEncode") 
	buffer[0] |= readPIRs();
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
	pirStat =  (PIND & PIR_SHOWER) >> 3; 
	pirStat |= (PIND & PIR_WC) >> 3;
	
//	Serial.print(digitalRead(4));
//	Serial.print(", ");
//	Serial.println(digitalRead(5));
//	Serial.println(pirStat);
	db("out readPIRs") 
	return pirStat;  //set bedSouth value to recieved data
}

void configure_fast(){
	db("in configure_fast") 
	pinMode(PIR_SHOWER, INPUT);
	pinMode(PIR_WC, INPUT);
//	DDRD &= !PIR_SHOWER;
//	DDRD &= !PIR_WC;
}
