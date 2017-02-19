#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include "printf.h"

#include "co2.h"
#include "localSensors.h"
#include "remoteNodes.h"
#include "config.h"
#include "humiditySensor.h"

#ifdef DEBUG
	#include "printf.h"
#endif

char commandBuffer[3];
uint8_t commandBuffer_Len = 0;

uint16_t slowData[SLOWDATA_SIZE] = {32767,32767,32767,32767,32767,32767,0,0,0};
uint16_t fastData[FASTDATA_SIZE];

RF24 reciever(pin::RADIO_CE, pin::RADIO_CS);
RF24* recieverPtr= &reciever;

RemoteNodes radio;
LocalSensors local;
TempHumid thSen;
Co2 co2;

RemoteNodes* radioPtr = &radio;
LocalSensors* localPtr = &local;
TempHumid* thPtr = &thSen;



////////////////////////////////////////////////////
////////////////////////////////////////////////////


  
void updateSlow_Local(){
  float result;
  
	//co2.rqCO2();FIXME
  //geather data from the local sensors
  result = thPtr->readTemperatureC();
  slowData[0] = int(result*100);
  result = thPtr->readHumidity(result);
  slowData[2] = int(result*100);
  
 // sensorData[4] = int(co2Sen.readCO2() );FIXME
}

bool slowDataComplete(){
	bool complete = true;
	for (unsigned int i =0; i < SLOWDATA_SIZE; i++){
	  if(slowData[i] == 32767){ //if the element is the default value not all
	    complete = false;//data has been collected and we are not rdy to send
	  }
	}
	return complete;
}

void sendFastData(){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled
  
  INTUNION_t toSend;
  
  //header that announces the data format
  Serial.write(headers::FAST_UPDATE);
//  Serial.println("");//FIXME
  for (unsigned int i = 0; i < FASTDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
	#ifdef DEBUG 
    Serial.println(fastData[i]);//FIXME //TODO do some ifdef debug here
	#endif
	#ifndef DEBUG 
    toSend.number = fastData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
	#endif
  }
  
  //reset sensorData to default values so we can easily check if it is complete
  memcpy(slowData, SLOWDATA_DEF, SLOWDATA_SIZE);
}

void sendSlowData(){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled
  
  INTUNION_t toSend;
  
  //header that announces the data format
  Serial.write(headers::SLOW_UPDATE);
//  Serial.println("");//FIXME
  for (unsigned int i = 0; i < SLOWDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
	#ifdef DEBUG 
    Serial.println(slowData[i]);//FIXME //TODO do some ifdef debug here
	#endif
	#ifndef DEBUG 
    toSend.number = slowData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
	#endif
  }
  
  //reset sensorData to default values so we can easily check if it is complete
  memcpy(slowData, SLOWDATA_DEF, SLOWDATA_SIZE);
}

void setup(){ 
  Serial.begin(9600); //Open serial connection to report values to host
	#ifdef DEBUG
	Serial.print("starting setup\n"); 	
	printf_begin();
	#endif

	radio.setup(fastData, slowData, recieverPtr);	
	local.setup(fastData);
	thSen.setup(pin::TERM_DATA, pin::TERM_CLOCK, radioPtr, localPtr);

  //give the pir sensor some time to calibrate
  delay(config::CALIBRATION_TIME);
	#ifdef DEBUG  
	Serial.print("setup done, starting response loop\n");
	#endif
  Serial.write(headers::SETUP_DONE);
}


void loop(){
  // serial read section
	while (Serial.available()){ // this will be skipped if no data present, leading to
                              // the code sitting in the delay function below
    delay(config::READSPEED);  //delay to allow buffer to fill //TODO check if really needed (should not be)
    if (Serial.available() >0)
    {
      int c = Serial.read(); //gets one byte from serial buffer
      if (c == 99){
        break;
      }
      commandBuffer[commandBuffer_Len] = c;
      commandBuffer_Len++;
    }
  }

  if (commandBuffer_Len >0) {
    switch(commandBuffer[0]){
      case 48: //acii 0
        updateSlow_Local();//requests the remote sensor values
        //and reads in the local sensors
        break;
      case 49: //acii 1
        //nothing         
        break;
      case 50: //acii 2
        break;
      case 51: //acii 3          
        break;
      case 52: //acii 4               
        break;
      default:
        //TODO replace with error code
        break;
    }//switch
  }//if
  commandBuffer_Len = 0;//empty the string

	//read local fast sensors  
	local.updateFast_Local();

  //read remote sensors
  radio.pollNodes();
 
  
  //check if all data has been collected
	if (slowDataComplete){sendSlowData();}
	Serial.print("sending some data");
	sendFastData();
  
  delay(config::RESETSPEED);
}
