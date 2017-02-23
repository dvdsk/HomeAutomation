#include "decode.h"

void checkSensorData(PirData* pirData){
  
  const unsigned char POLLING_FAST = 200;   //PIR and light Level
  const unsigned char POLLING_SLOW = 202;   //Temperature, humidity and co2
  
  INTUNION_t temp_bed, temp_bathroom, humidity_bed, humidity_bathroom;
  INTUNION_t co2, light_outside, light_bed, light_door, light_kitchen;
  
  uint32_t Tstamp;
  
  uint8_t pirDat[2];
  uint8_t fastData[2];//TODO change back to 10
  uint8_t slowData[10];      
  uint8_t toLog[18];   
  
  Serial arduino("/dev/ttyUSB0",115200);
  while (true){
    uint8_t x;
    x = arduino.readHeader();
    x = (int)x;
    switch (x) {      
      case POLLING_FAST:
				decodeFastData(Tstamp);   
        pirData->process(pirDat, Tstamp);
        break;        
      
      case POLLING_SLOW:
        
        arduino.readMessage(slowData, 10);
        std::cout << "got slow\n";          
        std::memcpy(temp_bed.bytes, slowData, 2);  
        std::memcpy(temp_bathroom.bytes, slowData+2, 2);  
        std::memcpy(humidity_bed.bytes, slowData+4, 2);  
        std::memcpy(humidity_bathroom.bytes, slowData+6, 2);
        std::memcpy(co2.bytes, slowData+8, 2);    
        
        //add last light data and send off for saving as binairy file
        std::memcpy(toLog, slowData, 10);
        std::memcpy(toLog+10, fastData+2, 8);          
        
      default:
        std::cout << "error no code matched, header: " << +x <<"\n";     
    }
  }
}

void decodeFastData(uint32_t Tstamp){
	uint8_t buffer[FASTDATA_SIZE];
	buffer = arduino.readMessage(fastData, FASTDATA_SIZE)

	//process movement values
	//if the there has been movement recently the value temp will be one this indicates that
	//movement[] needs to be updated for that sensor. Instead of an if statement we use multiplication 
	//with temp, as temp is either 1 or 0.
	for (int i = 0; i<5; i++){
		temp = (buffer[Idx::pirs] & (0b00001<<i)) & (buffer[Idx::pirs_updated] & (0b00001<<i))
		movement[i] = !temp * movement[i] + temp*Tstamp;
	}

	//process light values
	lightValues[lght::BED] = buffer[Idx::light_bed];
	lightValues_updated = true;	
}

void decodeSlowData(uint32_t Tstamp);
