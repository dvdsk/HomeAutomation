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
const int term_dataPin = 3; //PD3
const int term_clockPin = 2; //PD2

const short pirA = 0b00010000;
const short pirB = 0b00100000;

//radio address
static const uint8_t ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes

//radio commands
static const unsigned char NODE1_PIR = 1;
static const unsigned char NODE1_TEMP = 't';
static const unsigned char NODE1_TEMP_RESEND = 'd';


//default package
static const byte sendBuffer_def[5] = {255,0,0,0,0};
byte sendBuffer[5] = {255,0,0,0,0};

RF24 radio(7,8); //Set up nRF24L01 radio on SPI bus plus pins 7 & 8 (cepin, cspin)
TempHumid thSen (term_dataPin, term_clockPin);



void readPIRs(byte sendBuffer[5]){
  byte pirValues = 0 ;
  if ((PIND & pirA) != 0){ 
    pirValues = 0b00000100;//pir1 signal
    }
  else if ((PIND & pirB) != 0){
    pirValues = pirValues | 0b00001000;//pir2 signal
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
  radio.setChannel(108);// 2.508 Ghz - Above most Wifi Channels
  
  //no writing pipe needed since the ack goes back to the same adress as we are
  //reading on
  radio.openReadingPipe(1,ADDRESSES[1]);  // Open a reading pipe on address 0, pipe 1
  radio.startListening();                 // Start listening     

  radio.printDetails();                   // Dump the configuration of the rf unit for debugging

  //prepare hardware awk buffer
  byte sendBuffer[5] = {255,0,0,0,0};
  radio.writeAckPayload(1,sendBuffer, 5);//pre load pir values into into pipe 1
}

long debugCounter = 0;
void loop(void){
  byte gotByte;                          // Declare variables for the pipe & byte received
  float temp_raw, humidity_raw;
  INTUNION_t temp_c, humidity; 

  while (radio.available()){                   // Read all available payloads

    radio.read( &gotByte, 1 );
    radio.powerUp();                             //TODO does this fix radio going to low power mode?
                                                 // Respond directly with an ack payload.
//    debugCounter++;//TODO something to check with long term measurments
//    Serial.print("handling request:");
//    Serial.println(debugCounter);
    
    if (gotByte == NODE1_TEMP){
      //reads temp and sends response while also checking the PIR
      //and sending its status along
      temp_raw = thSen.readTemperatureC(readPIRs, sendBuffer, radio);
      humidity_raw = thSen.readHumidity(temp_raw, readPIRs, sendBuffer, radio);

      temp_c.number = int(temp_raw*100);
      humidity.number = int(humidity_raw*100);

      //copy the 2 bytes of each int into the sendbuffer
      memcpy(sendBuffer, temp_c.bytes, 2);
      memcpy(sendBuffer+2, humidity.bytes, 2);
      
      //add the latest PIR update
      readPIRs(sendBuffer);      

      Serial.println("");
      Serial.print("transmitting");
    }    
    else if (gotByte == NODE1_TEMP_RESEND){
      //send temperature records back together with new pir data with the next
      //transmission
      readPIRs(sendBuffer);
      Serial.print("."); 
    }    
    else if (gotByte == NODE1_PIR){
      //reset reset buffer send new pir data    
      memcpy(sendBuffer, sendBuffer_def, sizeof(sendBuffer_def));
      readPIRs(sendBuffer);
    }
    radio.writeAckPayload(1,sendBuffer, 5);
 }
}
