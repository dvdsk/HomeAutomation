//FOR REMOTE ARDUINO 1 (ARDUINO NANO), using termpin's 3 en 2 (data, sck) (PD3 en PD2)
#include <SPI.h>
#include "RF24.h"
#include "printf.h"
#include "humiditySensor.h"

typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

// Specify data and clock connections
static const int term_dataPin = 3; //PD3
static const int term_clockPin = 2; //PD2

static const short pirA = 0b00001000;//door
static const short pirB = 0b00000100;//krukje
static const short pirC = 0b00000010;//trashcan

//radio address
static const uint8_t ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes

//radio commands
static const unsigned char NODE2_PIR = 2;
static const unsigned char NODE2_LIGHT = 'l';
static const unsigned char NODE2_LIGHT_RESEND = 'i';

//default package
static const byte sendBuffer_def[5] = {255,0,0,0,0};
byte sendBuffer[5] = {255,0,0,0,0};

RF24 radio(7,8); //Set up nRF24L01 radio on SPI bus plus pins 7 & 8 (cepin, cspin)



void readPIRs(byte sendBuffer[5]){
  byte pirValues = 0 ;
  if ((PIND & pirA) != 0){ 
    pirValues = 0b00001000;//door
    }
  else if ((PIND & pirB) != 0){
    pirValues = pirValues | 0b00000100;//krukje
    }
  else if ((PIND & pirC) != 0){
    pirValues = pirValues | 0b00000010;//trashcan
    }
  pirValues = sendBuffer[4];
}

 

void setup(){

  Serial.begin(115200);
  // Setup and configure radio

  printf_begin();
  radio.begin();
  
  radio.setAddressWidth(3);               //sets adress with to 3 bytes long
  radio.setAutoAck(1);                    // Ensure autoACK is enabled
  radio.enableAckPayload();               // Allow optional ack payloads
  radio.setRetries(0,15);                 // Smallest time between retries, max no. of retries
  radio.setPayloadSize(5);                // Here we are sending 1-byte payloads to test the call-response speed
  
  //radio.setDataRate(RF24_250KBPS);
  
  radio.openWritingPipe(ADDRESSES[2]);    // Both radios on same pipes, but opposite addresses
  radio.openReadingPipe(1,ADDRESSES[0]);  // Open a reading pipe on address 0, pipe 1
  radio.startListening();                 // Start listening     

  radio.printDetails();                   // Dump the configuration of the rf unit for debugging

  //prepare hardware awk buffer
  byte sendBuffer[5] = {255,0,0,0,0};
  radio.writeAckPayload(1,sendBuffer, 5);//pre load pir values into into pipe 1
}

void loop(void){
  byte gotByte;                             // Declare variables for the pipe & byte received
  INTUNION_t light_door, light_kitchen;

    while (radio.available()){                  // Read all available payloads

      radio.read( &gotByte, 1 );
      radio.powerUp();//TODO does this fix radio going to low power mode?
           
    if (gotByte == NODE2_LIGHT){//l indicates a request for light level
      //transmission
      light_door.number = ;
      light_kitchen.number = ;
      
      memcpy(sendBuffer, light_door.bytes, 2);
      memcpy(sendBuffer+2, light_kitchen.bytes, 2);
      
      readPIRs(sendBuffer);
    }    
    else if (gotByte == NODE2_LIGHT_RESEND){
      readPIRs(sendBuffer);
    }
    
    else if (gotByte == NODE2_PIR){
      //reset temp part of buffer as it has been confirmed recieved    
      memcpy(sendBuffer, sendBuffer_def, sizeof(sendBuffer_def));
      readPIRs(sendBuffer);      
    }
    radio.writeAckPayload(1,sendBuffer, 5);
  }
}
