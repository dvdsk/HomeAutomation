#include <iostream>
#include "Serial.h"
#include "StoreData.h"

#include <typeinfo>//FIXME for debugging only

const unsigned char POLLING_FAST = 200;   //PIR and light Level
const unsigned char POLLING_SLOW = 202;   //Temperature, humidity and co2

typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

int main(int argc, char* argv[])
{
INTUNION_t temp_bed, temp_bathroom, humidity_bed, humidity_bathroom;
INTUNION_t co2, light_outside, light_bed, light_door, light_kitchen;
unsigned char pirData[2];
unsigned char fastData[10];
unsigned char slowData[10];      
unsigned char toLog[18];   

  try {
    Serial arduino("/dev/ttyUSB1",115200);
    StoreData log;

    std::cout << "doing stuff";

    while (true){
      unsigned char x;
      x = arduino.readHeader();
      x = (int)x;
//      std::cout << x << "\n"; 
      switch (x) {      
        case POLLING_FAST:


          arduino.readMessage(fastData);
          std::cout << "got fast\n";
          std::memcpy(pirData, fastData+0, 2);  //save PIR data
          
          std::memcpy(light_outside.bytes, fastData+2, 2);  
          std::memcpy(light_bed.bytes, fastData+4, 2);      
          std::memcpy(light_door.bytes, fastData+6, 2);  
          std::memcpy(light_kitchen.bytes, fastData+8, 2);
          
          //TODO analyse the pir data and log 'recent' (order of seconds) changes
          break;        
        
        case POLLING_SLOW:
          
          arduino.readMessage(slowData);
          std::cout << "got slow\n";          
          std::memcpy(temp_bed.bytes, slowData, 2);  
          std::memcpy(temp_bathroom.bytes, slowData+2, 2);  
          std::memcpy(humidity_bed.bytes, slowData+4, 2);  
          std::memcpy(humidity_bathroom.bytes, slowData+6, 2);
          std::memcpy(co2.bytes, slowData+8, 2);    
          
          //add last light data and send off for saving as binairy file
          std::memcpy(toLog, slowData, 10);
          std::memcpy(toLog+10, fastData+2, 8);          
          
          log.write(toLog);
          
        default:
          std::cout << "error no code matched\n";     
      }
    }
  } 
  
  catch(boost::system::system_error& e) {
    std::cout << "Error: "<< e.what() << std::endl;
    return 1;
  }
}
