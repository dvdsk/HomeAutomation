//FOR REMOTE ARDUINO 1 (ARDUINO NANO), using termpin's 3 en 2 (data, sck) (PD3 en PD2)
#include <SPI.h>
#include "RF24.h"
#include "humiditySensor.h"

typedef union
{
  float number;
  uint8_t bytes[4];
} FLOATUNION_t;

// Specify data and clock connections
const int term_dataPin = 3; //PD3
const int term_clockPin = 2; //PD2

const short pirA = 0b00010000;
const short pirB = 0b00100000;

//radio address
const byte ADDRESSES[][6] = {"1Node","2Node","3Node"};  // Radio pipe addresses

RF24 radio(7,8); //Set up nRF24L01 radio on SPI bus plus pins 7 & 8 (cepin, cspin)
TempHumid thSen (term_dataPin, term_clockPin);
                                                           // 




int readPIRs(){
  int pirValues = 0 ;
  if ((PIND & pirA) != 0){ 
    pirValues = pirValues | 1;//pir1 signal
    }
  else if ((PIND & pirB) != 0){
    pirValues = pirValues | 2;//pir2 signal
    }
  return pirValues;
}

void readAndSendPIRs(byte pipeNo){
  //check the PIR sensors then ack with the value
  char sendBuffer[9] = {-40,0,0,0,0,0,0,0,0};
  
  sendBuffer[8] = readPIRs();
  radio.writeAckPayload(pipeNo, sendBuffer, 9);
}
  

void setup(){

  Serial.begin(115200);
  // Setup and configure radio

  radio.begin();

  radio.enableAckPayload();                     // Allow optional ack payloads
  radio.enableDynamicPayloads();                // Ack payloads are dynamic payloads
  
  radio.openWritingPipe(ADDRESSES[0]);
  radio.openReadingPipe(1,ADDRESSES[1]);

  radio.startListening();                       // Start listening  
  
  readAndSendPIRs(1);
//  radio.writeAckPayload(1,&counter,1);          // Pre-load an ack-paylod into the FIFO buffer for pipe 1
  //radio.printDetails();
}


void loop(void){
  byte pipeNo, gotByte;                          // Declare variables for the pipe & byte received
  char sendBuffer[9] = {-40,0,0,0,0,0,0,0,0};
  FLOATUNION_t temp_c, humidity; 
  
  while (radio.available(&pipeNo)){              // Read all available payloads
    radio.read( &gotByte, 1 );                   
                                                 // Respond directly with an ack payload.
    if (gotByte == 't'){
      //reads temp and sends response while also checking the PIR
      //and sending its status along
      temp_c.number = thSen.readTemperatureC(readAndSendPIRs, radio);
      humidity.number = thSen.readHumidity(temp_c.number, readAndSendPIRs, radio);

      //copy the 4 bytes of each float into the sendbuffer
      memcpy(sendBuffer, temp_c.bytes, 4);
      memcpy(sendBuffer+4, humidity.bytes, 4);

      radio.writeAckPayload(pipeNo, sendBuffer, 9);
    }
    else{
      readAndSendPIRs(pipeNo);           
    }
 }

}
