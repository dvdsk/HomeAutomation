#include <Wire.h>
#include <SPI.h>
#include <RH_NRF24.h>

#include "humiditySensor.h"
#include "accelerometer.h"

// Specify data and clock connections
const int term_dataPin = 24; //PB2
const int term_clockPin = 22; //PB0

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
int accCounter = 0;
int accPeriod = 500;

//runs constructor to make opjebt test of class tempHumid from humiditySensor.h
TempHumid thSen (term_dataPin, term_clockPin);
Accelerometer acSen;
RH_NRF24 nrf24;

void readAcc(){
  acSen.readOut();
}

void readPIR(){
  //check the PIR sensor for movement as fast as possible, this happens
  //many many times a second
  
  //read registery of pin bank L (fast way to read state), 
  //returns byte on is high bit off is low. See this chart for which bit in the 
  //byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
  delay(1);//crashes if removed  
  if ((PINL & 1) != 0){
    Serial.print("m");
    }
  Serial.print("\n");
  }

void readLight(){
  //read light sensor (anolog) and return over serial, this happens many times
  //a second
  light = analogRead(light_signal);    // read the input pin
  Serial.print("l");//r to signal this is specially requested data
  Serial.print(light);
  Serial.print("\n");
}

void readTemp(){
  // Read values from the sensor, this function has a long sleep, we pass
  // funtions to it we want it to run to fill this sleep
  temp_c = thSen.readTemperatureC(readPIR,readLight,readAcc );
  humidity = thSen.readHumidity(temp_c, readPIR,readLight,readAcc );
  
  // Print the values to the serial port
  Serial.print("r");
  Serial.print("t");
  Serial.print(temp_c);
  Serial.print("h");
  Serial.print(humidity);
  Serial.print("\n");
  }

void readRoomSensors(){
  // Read temperature, humidity and light sensors and return there values over
  // serial
  temp_c = thSen.readTemperatureC(readPIR,readLight,readAcc );
  humidity = thSen.readHumidity(temp_c, readPIR,readLight,readAcc );
  light = analogRead(light_signal);    // read the input pin

  Serial.print("t");
  Serial.print(temp_c);
  Serial.print("h");
  Serial.print(humidity);
  Serial.print("l");  
  Serial.print(light);
  Serial.print("\n");
  }




void setup()
{ 
   Serial.begin(115200); // Open serial connection to report values to host
   
   //initialising and calibrating accelerometer
   acSen.setup();
   
   //initialise wireless communication net
     if (!nrf24.init())
    Serial.println("init failed");
    // Defaults after init are 2.402 GHz (channel 2), 2Mbps, 0dBm
    if (!nrf24.setChannel(1))
      Serial.println("setChannel failed");
    if (!nrf24.setRF(RH_NRF24::DataRate2Mbps, RH_NRF24::TransmitPower0dBm))
      Serial.println("setRF failed");
   
   //give the pir sensor some time to calibrate
   delay(2000);
   Serial.println("setup done, starting response loop");
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
          case 48: //acii 0
            readRoomSensors();
            break;
          case 49: //acii 1
            readTemp();            
            break;
          case 50: //acii 2
            accPeriod = 10;       
            break;
          case 51: //acii 3
            accPeriod = 500;               
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
//  readPIR();

  if (lightCounter > 10) {
//    readLight();  
    lightCounter = 0;
    }
  if (accCounter > accPeriod) {
    accCounter = 0;
//    acSen.readOut();
    }
  
  lightCounter++;
  accCounter++;
  
  delay(resetSpeed);
}
