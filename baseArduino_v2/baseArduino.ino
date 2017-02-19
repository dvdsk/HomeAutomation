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

//first element of slowdata used to check which values have been updated
uint16_t slowData[SLOWDATA_SIZE];
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
	co2.rqCO2();
	thSen.getTempHumid();
  co2.readCO2();
}

inline bool slowDataComplete(){	return (slowData[0] == SLOWDATA_COMPLETE);}

void sendFastData(){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled
  
	#ifdef DEBUG
  Serial.print("fastData: ");
	for (unsigned int i = 0; i < FASTDATA_SIZE; i++){
    Serial.print(fastData[i]);
		Serial.print("  ");	
	}
	Serial.print("\n");	
	#endif
	#ifndef DEBUG
  INTUNION_t toSend;
  Serial.write(headers::FAST_UPDATE);
  for (unsigned int i = 0; i < FASTDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
    toSend.number = fastData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
	}	
	#endif
}

void sendSlowData(){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled
  
	#ifdef DEBUG
  Serial.print("slowData: ");
	for (unsigned int i = 0; i < SLOWDATA_SIZE; i++){
    Serial.print(slowData[i]);
		Serial.print("  ");	
	}
	Serial.print("\n");	
	#endif
	#ifndef DEBUG
  INTUNION_t toSend;
  Serial.write(headers::FAST_UPDATE);
  for (unsigned int i = 0; i < SLOWDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
    toSend.number = slowData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
	}	
	#endif
}

void setup(){ 
  Serial.begin(9600); //Open serial connection to report values to host
	#ifdef DEBUG
	Serial.print("starting setup\n"); 	
	printf_begin();
	#endif

	radio.setup(fastData, slowData, recieverPtr);	
	local.setup(fastData);
	thSen.setup(pin::TERM_DATA, pin::TERM_CLOCK, radioPtr, localPtr, slowData);
	co2.setup(slowData);

	slowData[0] = 0;
		
	Serial.println(SLOWDATA_COMPLETE);

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
	if(slowDataComplete()){
		Serial.print("sending slowdata"); 
		sendSlowData();
		slowData[0] = 0;//set slowdata to incomplete again.	
	}
	
	sendFastData();
  
  delay(config::RESETSPEED);
}
