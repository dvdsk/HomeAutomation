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
const int resetSpeed = 1000; //time for the connection to reset
const int calibrationTime = 2000; //setup wait period

const byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 

int buffer[3];
int bufferLen = 0;
int lightCounter = 0;
int accCounter = 0;
int accPeriod = 5;


//runs constructor to make opjebt test of class tempHumid from humiditySensor.h
TempHumid thSen (term_dataPin, term_clockPin);
Accelerometer acSen;

//needed for passing function to a class, dont know why its needed though..
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
  int light;
  
  light = analogRead(light_signal);    // read the input pin
  Serial.print("l");//r to signal this is specially requested data
  Serial.print(light);
  Serial.print("\n");
}

void readTemp(){
  // Read values from the sensor, this function has a long sleep, we pass
  // funtions to it we want it to run to fill this sleep
  int temp_c;
  int humidity;
  
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

int readCO2(Stream& sensorSerial){
  //reads awnser from Co2 sensor that resides in the hardware serial buffer
  //this can be called some time after reqeusting the data thus it is not 
  //needed to wait for a reply after the request, Will also not 
  //block if no reply present but return -1 instead

  char response[9]; //changed was char, byte is better prevents issue with char
                    //in reading out the array in hex (like values such as 
                    //FFFFFFF86 instead of 86) //again to char since makefile
                    //wants it to be char really badly
  int responseHigh;
  int responseLow;
  int ppm;
  
  while (sensorSerial.available() > 8){
    if (!sensorSerial.read() == 0XFF){//check if reply from sensor in buffer
      sensorSerial.read();
    }
    else{
      sensorSerial.readBytes(response, 8);

      responseHigh = (int) response[1];
      responseLow = (int) response[2];
      ppm = (256*responseHigh)+responseLow;
      return ppm;
    }
  }
  return -1;
}    

void readRoomSensors(){
  // Read temperature, humidity and light sensors and return there values over
  // serial
  int ppmCO2;
  int light;
  float humidity;
  float temp_c;
  
  
  Serial1.write(REQUESTCO2,9);// request the CO2 sensor to do a reading
  temp_c = thSen.readTemperatureC(readPIR,readLight,readAcc );
  humidity = thSen.readHumidity(temp_c, readPIR,readLight,readAcc );
  light = analogRead(light_signal);    // read the input pin
  ppmCO2 = readCO2(Serial1);// retrieve the reading from the CO2 sensor

  Serial.print("t");
  Serial.print(temp_c);
  Serial.print("h");
  Serial.print(humidity);
  Serial.print("l");  
  Serial.print(light);
  Serial.print("c");  
  Serial.print(ppmCO2);
  Serial.print("\n");
  }




void setup()
{ 
   Serial.begin(115200); //Open serial connection to report values to host
   Serial1.begin(9600);  //Opens the second serial port with a baud of 9600 
                         //connect TX from MH Co2 sensor to TX1 on arduino etc
   
   //initialising and calibrating accelerometer
   acSen.setup();
      
   //give the pir sensor some time to calibrate
   delay(calibrationTime);
   Serial.print("setup done, starting response loop\n");
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
            accPeriod = 1;  //fast polling    
            break;
          case 51: //acii 3
            accPeriod = 5;  //slow polling             
            break;
          case 52: //acii 4               
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
  readPIR(); 


  if (lightCounter > 10) {
    readLight(); 
    lightCounter = 0;
    }
  if (accCounter > accPeriod) {
    accCounter = 0;
    acSen.readOut();
    }
    
  lightCounter++;
  accCounter++;
  
  delay(resetSpeed);
}
