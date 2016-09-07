#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include "printf.h"


//the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways
typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

// Specify data and clock connections
static const int term_dataPin = 24; //PB2
static const int term_clockPin = 22; //PB0

// Specify signal connections
static const int pir_signal = 49;
static const int light_signal = 0; //anolog

// Radio connections
static const int CEPIN = 7;
static const int CSPIN = 9;
static const uint8_t ADDRESSES[][4] = { "No1", "No2" }; // Radio pipe addresses 3 bytes
//is the absolute minimum


static const signed short int SENSORDATA_SIZE = 9;
static const signed short int sensorData_def[SENSORDATA_SIZE] = {32767,32767,32767,32767,32767,32767,0,0,0};
signed short int sensorData[SENSORDATA_SIZE] = {32767,32767,32767,32767,32767,32767,0,0,0};
//initialised as 32767 for every value stores: temp_bed, temp_bathroom, 
//humidity_bed, humidity_bathroom, co2, light_bed, light_outside, light_door, 
//light_kitchen, whenever data is send we reset to this value


//runs constructor to make opjebt test of class tempHumid from humiditySensor.h
RF24 radio(CEPIN,CSPIN); //Set up nRF24L01 radio on SPI bus plus cepin, cspin

//needed for passing function to a class, dont know why its needed though..
void checkWirelessNodes(){
  //ask wireless node(s) (currently 1 implemented) for a status update
  //process status data
  
  byte rcbuffer[5];
  static const byte rqUpdate[1] = {1};
  radio.write(rqUpdate, 1 ); //write 1 to the currently opend writingPipe
  if(radio.available() ){
    radio.read( &rcbuffer, 5 );
    //check if temperature data is present
    if (rcbuffer[0] == 255){
//        Serial.println("PIRDATA");
    }
    else{//temperature must be contained in the first 2 bytes
        Serial.println("TEMPDATA");
    }
  }
}

void setup()
{ 
  Serial.begin(115200); //Open serial connection to report values to host
  printf_begin();
  delay(2000);
 
  //initialise radio
  radio.begin();

  radio.setAddressWidth(3);               //sets adress with to 3 bytes long
  radio.setAutoAck(1);                    // Ensure autoACK is enabled
  radio.enableAckPayload();               // Allow optional ack payloads
  radio.setRetries(0,15);                 // Smallest time between retries, max no. of retries
  radio.setPayloadSize(5);                // Here we are sending 1-byte payloads to test the call-response speed
  
  radio.openWritingPipe(ADDRESSES[0]);    // Both radios on same pipes, but opposite addresses
  radio.openReadingPipe(1,ADDRESSES[1]);  // Open a reading pipe on address 1, pipe 1
  radio.startListening();                 // Start listening
  radio.printDetails();                   // Dump the configuration of the rf unit for debugging
    
  //give the pir sensor some time to calibrate
  Serial.print("setup done, starting response loop\n");
  radio.stopListening();
}


void loop(){
  for(int i =0 ; i<400; i++){
    checkWirelessNodes();
  }
    static const byte rqUpdate[1] = {'t'};
    radio.write(rqUpdate, 1 ); //write 1 to the currently opend writingPipe
    Serial.print("requesting temp data");
}
