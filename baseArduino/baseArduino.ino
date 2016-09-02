#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include "printf.h"

#include "humiditySensor.h"
#include "accelerometer.h"

typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

// Specify data and clock connections
const int term_dataPin = 24; //PB2
const int term_clockPin = 22; //PB0

// Specify signal connections
const int pir_signal = 49;
const int light_signal = 0; //anolog

// Radio connections
const int CEPIN = 7;
const int CSPIN = 9;
const uint8_t ADDRESSES[][4] = { "No1", "No2" }; // Radio pipe addresses 3 bytes
//is the absolute minimum

// script setup parameters
const int readSpeed = 1; //time between reading individual chars
const int debugSpeed = 0; //time between reading and reply-ing used for debug
const int resetSpeed = 2000; //time for the connection to reset
const int calibrationTime = 2000; //setup wait period

const byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 

int buffer[3];
int bufferLen = 0;
int lightCounter = 0;
int accCounter = 0;
int accPeriod = 200;

//For that, the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways


//runs constructor to make opjebt test of class tempHumid from humiditySensor.h
TempHumid thSen (term_dataPin, term_clockPin);
Accelerometer acSen;
RF24 radio(CEPIN,CSPIN); //Set up nRF24L01 radio on SPI bus plus cepin, cspin

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
//TODO renable when debugging done
//    Serial.print("m");
    }
//  Serial.print("\n");
  }

void remotePIR(){
  
  char recieveBuffer[9];
  const char RQ_PIR[1] = {'p'};
  
  //request pirData and wait at most 4 millisec for reply
  radio.write(RQ_PIR,1);
  for(int i= 0; i < 4; ++i) {
    delay(1);// TODO [OPTIMISE] check if 1 is really needed
    if (radio.available()) {
      radio.read( &recieveBuffer, 9 );
      //unpack pir data on python base
      Serial.print("rm");
      Serial.print(buffer[0]);
      break;
      }
    }
  }

void readLight(){
  //read light sensor (anolog) and return over serial, this happens many times
  //a second
  int light;
  
  light = analogRead(light_signal);    // read the input pin
//TODO renable when debugging done
//  Serial.print("l");//r to signal this is specially requested data
//  Serial.print(light);
//  Serial.print("\n");
}

void readTemp(){
  // Read values from the sensor, this function has a long sleep, we pass
  // funtions to it we want it to run to fill this sleep
  int temp_c; //FIXME this does not work
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


void remoteTemp(float &rtemp_c, float &rhumidity, void (*f1)(void), void (*f2)(void), void (*f3)(void)){
  
  byte rcBuffer[5];
  const char RQ_TEMP[1] = {'t'};
  const char RQ_PIR[1]= {'p'};
  INTUNION_t temp_c, humidity;
  
  //request data and wait for reply
  radio.write(RQ_TEMP,1);
  for(int i= 0; i < 10; ++i) {
    delay(1000);// FIXME
    //instead of using the above delay (and wasting cycles) we run readPir
    //and other functions
//    f1(); //readpir 
//    f2(); //readlight   //TODO re-enable these  
    //Possibility for an f3() here but it is currently not used

//    Serial.print("checking radio");
    if (radio.available()) {
      radio.read( &rcBuffer, 5 );
      Serial.print(rcBuffer[0]);
      if (rcBuffer[0] != 255){ //TODO rewrite voor 12 bit data structure
          memcpy(temp_c.bytes, rcBuffer, 2); 
          memcpy(humidity.bytes, rcBuffer+2, 2); //copy from buffer[2] t/m buffer[3]
          
          Serial.print("rcBuffer: \n");
          Serial.print(rcBuffer[0],HEX);
          Serial.print(rcBuffer[1],HEX);
          Serial.print("\n");
          Serial.print(rcBuffer[2],HEX);
          Serial.print(rcBuffer[3],HEX);
          Serial.print("\n");
          Serial.print(rcBuffer[4],HEX);
          Serial.print("\ndone: \n");
          
          
          Serial.print("recieved data:");
          Serial.print("\n");
          Serial.print(temp_c.number);
          Serial.print("\n");
          Serial.print(humidity.number);
          Serial.print("\n");
          
          rtemp_c = float(temp_c.number)/100;//set to the int representation of the 2 byte array
          rhumidity = float(humidity.number)/100;//TODO remove this (do this while rewriting for
          //Serial.write ipv Serial.print
          
          break;
      }
      else{//let analysis of this value be done on the arduino
//        Serial.print("rm");
//        Serial.print(buffer[0]);//TODO undo comment
        radio.write(RQ_PIR, 1);
      }
    }
  }
}
  

void readRoomSensors(){
  // Read temperature, humidity and light sensors and return there values over
  // serial
  int ppmCO2;
  int light;
  float humidity, rHumidity;
  float temp_c, rTemp_c;
  
  Serial1.write(REQUESTCO2,9);// request the CO2 sensor to do a reading
  temp_c = thSen.readTemperatureC(readPIR,readLight,readAcc );
  humidity = thSen.readHumidity(temp_c, readPIR,readLight,readAcc );
  light = analogRead(light_signal); // read the input pin
  ppmCO2 = readCO2(Serial1);// retrieve the reading from the CO2 sensor
  
  //request remote sensor values
  Serial.print("reading remote temperature\n");
  remoteTemp(rTemp_c, rHumidity, readPIR,readLight,readAcc);
    
  //TODO rewrite for Serial.write()
  Serial.print("rt");
  Serial.print(rTemp_c);
  Serial.print("rh");
  Serial.print(rHumidity);
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
  printf_begin();
  delay(2000);
 
  //initialising and calibrating accelerometer
  acSen.setup();
 
  //initialise radio
  Serial.print("setting up radio\n");
  radio.begin();

  radio.setAddressWidth(3);               //sets adress with to 3 bytes long
  radio.setAutoAck(1);                    // Ensure autoACK is enabled
  radio.enableAckPayload();               // Allow optional ack payloads
  radio.setRetries(0,15);                 // Smallest time between retries, max no. of retries
  radio.setPayloadSize(9);                // Here we are sending 1-byte payloads to test the call-response speed
  
  radio.openWritingPipe(ADDRESSES[0]);  // Both radios on same pipes, but opposite addresses
  radio.openReadingPipe(1,ADDRESSES[1]);// Open a reading pipe on address 0, pipe 1
  radio.startListening();                 // Start listening
  radio.printDetails();                   // Dump the configuration of the rf unit for debugging
    
  //give the pir sensor some time to calibrate
  delay(calibrationTime);
  Serial.print("setup done, starting response loop\n");
}

byte counter2 = 'p';
void debugWireless(){
  radio.stopListening();                                  // First, stop listening so we can talk.
      
  printf("Now sending %d as payload. ",counter2); 
  byte gotBytes[5]; 
  unsigned long time = micros();                          // Take the time, and send it.  This will block until complete   
                                                          //Called when STANDBY-I mode is engaged (User is finished sending)
  if (!radio.write( &counter2, 1 )){
    Serial.println(F("failed."));      
  }else{
    if(!radio.available()){ 
      Serial.println(F("Blank Payload Received.")); 
    }else{
      while(radio.available() ){
        unsigned long tim = micros();
        radio.read( &gotBytes, 5 );
        printf("Got response %d, round-trip delay: %lu microseconds\n\r",gotBytes[0],tim-time);
        Serial.print(gotBytes[1]);
        counter2++;
      }
    }
  }
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
            accPeriod = 200;  //slow polling             
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
/*  readPIR(); */


  if (lightCounter > 10) {
/*    readLight(); */
    lightCounter = 0;
    }
  if (accCounter > accPeriod) {
    accCounter = 0;
/*    acSen.readOut();*/
    accCounter = 0;
    }
    
  lightCounter++;
  accCounter++;
  
  debugWireless();
  delay(resetSpeed);
}
