/*
   Dec 2014 - TMRh20 - Updated
   Derived from examples by J. Coliz <maniacbug@ymail.com>
*/
#include <SPI.h>
#include "RF24.h"

static const int CEPIN = 7;
static const int CSPIN = 9;
static const uint8_t ADDRESSES[][4] = { "No1", "No2" }; // Radio pipe addresses 3 bytes

RF24 radio(CEPIN,CSPIN); //Set up nRF24L01 radio on SPI bus plus cepin, cspin

                                                                           // Topology
byte addresses[][6] = {"1Node","2Node"};              // Radio pipe addresses for the 2 nodes to communicate.

byte counter = 1;                                                          // A single byte to keep track of the data being sent back and forth
double arrivedCounter = 0;
double failureCounter = 0;
long unsigned int time;
long unsigned int startTime; 
long unsigned int TIME_PER_CHANNEL = 1000; //milliseconds
long unsigned int channel = 1;
bool notdone = true;

void setup(){
  Serial.begin(115200);
  radio.begin();

  radio.setAddressWidth(3);               //sets adress with to 3 bytes long
  radio.setAutoAck(1);                    // Ensure autoACK is enabled
  radio.enableAckPayload();               // Allow optional ack payloads
  radio.setRetries(0,15);                 // Smallest time between retries, max no. of retries
  radio.setPayloadSize(5);                // Here we are sending 1-byte payloads to test the call-response speed
  
  //radio.setDataRate(RF24_250KBPS);
  radio.setChannel(108);// 2.508 Ghz - Above most Wifi Channels
  
  radio.openWritingPipe(ADDRESSES[0]);    // Both radios on same pipes, but opposite addresses
  radio.openReadingPipe(1,ADDRESSES[1]);  // Open a reading pipe on address 1, pipe 1
  radio.startListening();                 // Start listening
  radio.printDetails();                   // Dump the configuration of the rf unit for debugging
  radio.stopListening();

  startTime = millis();
}
void loop(void) {
    byte gotByte[5]; // Initialize a variable for the incoming response
      
/*    Serial.print(F("Now sending ")); // Use a simple byte counter as payload*/
/*    Serial.println(counter);*/
    
/*    if (millis()-startTime > channel*TIME_PER_CHANNEL && notdone){*/
/*        Serial.print("Channel ");*/
/*        Serial.print(channel);*/
/*        Serial.print(" Failure rate: ");*/
/*        Serial.print(100*failureCounter/arrivedCounter);*/
/*        Serial.println("%");*/
/*        failureCounter = 0;*/
/*        arrivedCounter = 0;*/
/*        channel++;*/
/*        if (channel > 128){notdone = false;}    */
/*    }*/

/*    unsigned long time = micros();  // Record the current microsecond count   */
                                                            
    if (radio.write(&counter,1) ){  // Send the counter variable to the other radio 
        if(!radio.available()){     // If nothing in the buffer, we got an ack but it is blank
/*            Serial.print(F("Got blank response. round-trip delay: "));*/
/*            Serial.print(micros()-time);*/
/*            Serial.println(F(" microseconds"));     */
        }else{      
            while(radio.available() ){  // If an ack with payload was received
                radio.read( &gotByte, 5 ); // Read it, and display the response time
/*                unsigned long timer = micros();*/
                
/*                Serial.print(F("Got response "));*/
/*                Serial.print(gotByte[0]);*/
/*                Serial.print(F(" round-trip delay: "));*/
/*                Serial.print(timer-time);*/
/*                Serial.println(F(" microseconds"));*/
                counter++; // Increment the counter variable
                arrivedCounter++;
            }
        }
    
    }else{
        failureCounter++;
        Serial.print(" Failure rate: ");
        Serial.println(100*failureCounter/(failureCounter+arrivedCounter));
        } // If no ack response, sending failed
    
    delay(0);  // Try again later
}

