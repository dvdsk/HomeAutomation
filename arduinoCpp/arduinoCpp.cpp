#include <Wire.h>

#include "humiditySensor.h"
#include "accelerometer.h"

// Specify data and clock connections
const int term_dataPin = 51; //PB2
const int term_clockPin = 53; //PB0
//const int term_vcc = 52; //PB1

const int light_vcc = 50;

// Specify signal connections
const int pir_signal = 49;
const int light_signal = 0; //anolog

// script setup parameters
const int readSpeed = 1; //time between reading individual chars
const int debugSpeed = 0; //time between reading and reply-ing used for debug
const int resetSpeed = 1; //time for the connection to reset
const int calibrationTime = 2; //setup wait period


float temp_c;
float humidity;
int light;
int buffer[3];
int bufferLen = 0;
int lightCounter = 0;
int accCounter = 100;
int accPeriod = 10;


//runs constructor to make opjebt test of class tempHumid from humiditySensor.h
TempHumid thSen (term_dataPin, term_clockPin);
//Accelerometer acSen;


void setup()
{ 
   Serial.begin(9600); // Open serial connection to report values to host
   Serial.println("Starting up");
     
   //give the pir sensor some time to calibrate
   delay(2000);
   Serial.println("Done with startup");
}


void readTemp(){
  // Read values from the sensor
  temp_c = thSen.readTemperatureC();
  humidity = thSen.readHumidity();
  
  // Print the values to the serial port
  Serial.print("r");
  Serial.print("t");
  Serial.print(temp_c);
  Serial.print("h");
  Serial.print(humidity);
  Serial.print("\n");
  }
  

void readLight(){
    light = analogRead(light_signal);    // read the input pin
    Serial.print("l");//r to signal this is specially requested data
    Serial.print(light);

/*    Serial.print("\n"); */
}

void readRoomSensors(){
  // Read values from the sensor
  temp_c = thSen.readTemperatureC();
  humidity = thSen.readHumidity();    
  light = analogRead(light_signal);    // read the input pin

  Serial.print("t");
  Serial.print(temp_c);
  Serial.print("h");
  Serial.print(humidity);
  Serial.print("l");  
  Serial.print(light);
  Serial.print("\n");
  }


//void readAcc(){
//  Serial.print("started bad sensors readout speed");
//  acSen.readOut();
//}


void setup()
{ 
   Serial.begin(9600); // Open serial connection to report values to host
   delay(10000);
   Serial.println("Starting up");
     
   //give the pir sensor some time to calibrate
   delay(2000);
}


void loop(){
  // serial read section
  while (Serial.available()){ // this will be skipped if no data present, leading to
                              // the code sitting in the delay function below
    delay(readSpeed);  //delay to allow buffer to fill 
    if (Serial.available() >0)
    {
      int c = Serial.read(); //gets one byte from serial buffer
      if (c == 99){
        break;
      }
      buffer[bufferLen] = c;
      bufferLen++;
    }
  }

  if (bufferLen >0) {
    switch(buffer[0]) {
      case 48:
        switch(buffer[1]){
          case 48:
            readRoomSensors();
            break;
          case 49:
            readTemp();            
            break;
          case 50:
            accPeriod = 10;            
            break;
          default:
            Serial.print("error not a sensor\n");
            break;
        }//switch
        break;
      case 49:
        Serial.print("doing motor shit\n");   
        break;   
      default:
        Serial.print("error not a sensor/motor command\n");
        break;
    }    
  }//if

  bufferLen = 0;//empty the string*/
  thSen.readPIR();

  if (lightCounter > 10) {
    readLight();  
    lightCounter = 0;
    }
  if (accCounter > accPeriod) {
    accCounter = 0;
    //readAcc();
    }
  
  lightCounter++;
  accCounter++;
  
  delay(resetSpeed);
}
