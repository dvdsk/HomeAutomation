/*
   Dec 2014 - TMRh20 - Updated
   Derived from examples by J. Coliz <maniacbug@ymail.com>
*/
/**
 * Example for efficient call-response using ack-payloads 
 * 
 * This example continues to make use of all the normal functionality of the radios including 
 * the auto-ack and auto-retry features, but allows ack-payloads to be written optionlly as well. 
 * This allows very fast call-response communication, with the responding radio never having to 
 * switch out of Primary Receiver mode to send back a payload, but having the option to switch to 
 * primary transmitter if wanting to initiate communication instead of respond to a commmunication. 
 */
 
#include <SPI.h>
#include "RF24.h"


/* Hardware configuration: Set up nRF24L01 radio on SPI bus plus pins 7 & 8 */
RF24 radio(7,8); //cepin, cspin
/**********************************************************/
                                                                           // Topology
byte addresses[][6] = {"1Node","2Node"};              // Radio pipe addresses for the 2 nodes to communicate.

byte counter = 1;                                                          // A single byte to keep track of the data being sent back and forth


void setup(){

  Serial.begin(115200);
  Serial.println(F("RF24/examples/GettingStarted_CallResponse"));
 
  // Setup and configure radio

  radio.begin();

  radio.enableAckPayload();                     // Allow optional ack payloads
  radio.enableDynamicPayloads();                // Ack payloads are dynamic payloads

  radio.openWritingPipe(addresses[1]);        // Both radios listen on the same pipes by default, but opposite addresses
  radio.openReadingPipe(1,addresses[0]);      // Open a reading pipe on address 0, pipe 1
  
  radio.startListening();                       // Start listening  
  
  radio.writeAckPayload(1,&counter,1);          // Pre-load an ack-paylod into the FIFO buffer for pipe 1
  //radio.printDetails();
}






void loop(void) {
  byte gotByte;                                           // Initialize a variable for the incoming response

  radio.stopListening();                                  // First, stop listening so we can talk.      
  Serial.print(F("Now sending "));                         // Use a simple byte counter as payload
  Serial.println(counter);

  unsigned long time = micros();                          // Record the current microsecond count   
                                                          
  if ( radio.write(&counter,1) ){                         // Send the counter variable to the other radio 
      if(!radio.available()){                             // If nothing in the buffer, we got an ack but it is blank
          Serial.print(F("Got blank response. round-trip delay: "));
          Serial.print(micros()-time);
          Serial.println(F(" microseconds"));     
      }else{      
          while(radio.available() ){                      // If an ack with payload was received
              radio.read( &gotByte, 1 );                  // Read it, and display the response time
              unsigned long timer = micros();
              
              Serial.print(F("Got response "));
              Serial.print(gotByte);
              Serial.print(F(" round-trip delay: "));
              Serial.print(timer-time);
              Serial.println(F(" microseconds"));
              counter++;                                  // Increment the counter variable
          }
      }

  }else{        Serial.println(F("Sending failed.")); }          // If no ack response, sending failed
  delay(1000);  // Try again later

}
