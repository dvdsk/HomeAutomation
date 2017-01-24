#include <iostream>
#include <typeinfo>//FIXME for debugging only
#include <ctime>

#include <math.h>       /* sin */ //FIXME for debugging only

#include "Serial.h"
#include "dataStorage/MainData.h"
#include "dataStorage/PirData.h"
#include "graph/MainGraph.h"

#include <signal.h>
#include <boost/exception/diagnostic_information.hpp> //for debugging

const std::string PATHPIR = "pirs.binDat";
const int CACHESIZE_pir = 8;
const int CACHESIZE_slowData = 22;

//cache for data
uint8_t cache1[CACHESIZE_pir];
uint8_t cache2[CACHESIZE_slowData];
//uint8_t cache3[CACHESIZE_pir];

FILE* file1; //needed as global for interrupt handling
FILE* file2;
FILE* file3;

typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;


void interruptHandler(int s){

  fflush(file1);
  fflush(file2);
  fflush(file3);
  printf("Caught signal %d\n",s);
  exit(1); 
}

uint32_t unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

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

        arduino.readMessage(fastData, 2);//TODO 2 to 10
        Tstamp = unix_timestamp();
        
        std::cout << "got: " << +fastData[0]<<" " << +fastData[1] << "\n";
        std::memcpy(pirData, fastData+0, 2);  //save PIR data
        
        std::memcpy(light_outside.bytes, fastData+2, 2);  
        std::memcpy(light_bed.bytes, fastData+4, 2);      
        std::memcpy(light_door.bytes, fastData+6, 2);  
        std::memcpy(light_kitchen.bytes, fastData+8, 2);
        
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

void debug(PirData& pirData, SlowData& slowData){
  uint32_t Tstamp;
  uint8_t pirDat[2];
  uint8_t slowDat[9];
  uint16_t temp;

  ////INPUT FAKE PIR DATA:
  //Tstamp = 1481496152;
  //for(uint32_t i=Tstamp; i<Tstamp+100000000; i+=10){

    //pirDat[0] = 0b00000000;
    //pirDat[1] = 0b11111111;  
    //pirData.process(pirDat, i);
    
    //pirDat[0] = 0b11111111;
    //pirDat[1] = 0b11111111;  
    //pirData.process(pirDat, i+5);  
  //}

  ////INPUT FAKE TEMP DATA:
  //Tstamp = 1481496152;
  //for(uint32_t i=Tstamp; i<Tstamp+100000000; i+=10){
    //temp = (uint16_t)(sin(i/40.0)*100+100);
    //slowDat[0] = (uint8_t)temp;
    //slowDat[1] = (uint8_t)(temp >> 8);
    //slowData.process(slowDat, i);
  //}
  
  //std::vector<plotables> toPlot = {TEMP_BED, MOVEMENTSENSOR0};
  //std::vector<plotables> toPlot = {TEMP_BED};
  std::vector<plotables> toPlot = {HUMIDITY_BED};
  Graph graph(toPlot, 1481496152, 1481496152+1000, pirData, slowData);
}

int main(int argc, char* argv[])
{
  PirData pirData("pirs", cache1, CACHESIZE_pir);
  SlowData slowData("slowData", cache2, CACHESIZE_slowData);
  file1 = pirData.getFileP();
  file2 = slowData.getFileP();
  
  signal(SIGINT, interruptHandler);  
  //checkSensorData(&pirData);
  debug(pirData, slowData);


  
}
