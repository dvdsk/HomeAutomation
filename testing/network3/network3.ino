/*
   Dec 2014 - TMRh20 - Updated
   Derived from examples by J. Coliz <maniacbug@ymail.com>
*/
#include <SPI.h>
#include "RF24.h"


RF24 radio(7,8);

static const uint8_t ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes

byte counter = 1;                                                          // A single byte to keep track of the data being sent back and forth
void setup(){
  Serial.begin(115200);
  radio.begin();
  
  radio.setAddressWidth(3);               //sets adress with to 3 bytes long
  radio.setAutoAck(1);                    // Ensure autoACK is enabled
  radio.enableAckPayload();               // Allow optional ack payloads
  radio.setRetries(0,1);                  // Smallest time between retries, max no. of retries
  radio.setPayloadSize(15);               // Here we are sending 1-byte payloads to test the call-response speed
  
  //radio.setDataRate(RF24_250KBPS);
  radio.setChannel(108);// 2.508 Ghz - Above most Wifi Channels
  
  //no writing pipe needed since the ack goes through to the readingpipe
  radio.openReadingPipe(1,ADDRESSES[1]);  // Open a reading pipe on address 0, pipe 1
  radio.startListening();                 // Start listening     

  radio.printDetails();                   // Dump the configuration of the rf unit for debugging

  //prepare hardware awk buffer
  byte sendBuffer[5] = {0,255,255,255,255};
  radio.writeAckPayload(1,sendBuffer, 5);//pre load pir values into into pipe 1
}


void loop(void) {
    byte sendBuffer[5] = {0,255,29,210,234};
    byte pipeNo, gotByte;// Declare variables for the pipe and the byte received
    while( radio.available(&pipeNo)){ // Read all available payloads
      radio.read( &gotByte, 1 );                   
      // Since this is a call-response. Respond directly with an ack payload.
      sendBuffer[0] = gotByte + 1; // Ack payloads are much more efficient than switching 
      //to transmit mode to respond to a call
      radio.writeAckPayload(pipeNo,sendBuffer, 5 );  //This can be commented out to send empty payloads.
      Serial.print(F("Loaded next response "));
      Serial.println(gotByte);  
   }
}

