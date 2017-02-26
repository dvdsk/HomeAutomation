#include "config.h"

char commandBuffer[3];
uint8_t commandBuffer_Len = 0;

//first element of slowdata used to check which values have been updated
uint16_t slowData[SLOWDATA_SIZE];
uint16_t fastData[FASTDATA_SIZE];

////////////////////////////////////////////////////
////////////////////////////////////////////////////


  
void updateSlow_Local(){
	//co2.rqCO2();
	//thSen.getTempHumid();
	slowData[Idx::UPDATED] |= (1 << Idx::TEMPERATURE_BED) | (1<<Idx::HUMIDITY_BED);	
	slowData[Idx::TEMPERATURE_BED] = 200;
	slowData[Idx::HUMIDITY_BED] = 201;
  //co2.readCO2();
	slowData[Idx::CO2] = 500;
	slowData[Idx::UPDATED] |= 1 << Idx::CO2; //indicate co2 has been updated
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

  Serial.write((uint8_t)fastData[0]);
	for (unsigned int i = 1; i < FASTDATA_SIZE-2; i++){
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
	for (unsigned int i = 1; i < SLOWDATA_SIZE; i++){
    Serial.print(slowData[i]);
		Serial.print("  ");	
	}
	Serial.print("\n");	
	#endif
	#ifndef DEBUG
  INTUNION_t toSend;
  Serial.write(headers::FAST_UPDATE);
	//i=1 as we dont want to send the completeness info
  for (unsigned int i = 1; i < SLOWDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
    toSend.number = slowData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
	}	
	#endif
}

void setup(){ 
  Serial.begin(9600); //Open serial connection to report values to host
  Serial.write(headers::STARTUP_DONE);

	slowData[0] = 0;
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
	///////local.updateFast_Local();///////////
	fastData[Idx::PIRS] = 0b01010000;
	fastData[Idx::LIGHT_BED] = 100;

  //read remote sensors
  //radio.pollNodes(); //TODO does nothing at the moment
 
  
  //check if all data has been collected
	if(slowDataComplete()){
		Serial.print("sending slowdata"); 
		sendSlowData();
		slowData[0] = 0;//set slowdata to incomplete again.	
	}
	
	sendFastData();
  
  delay(config::RESETSPEED);
}
