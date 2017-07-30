#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include <printf.h>



//
void handle_fast();
void handle_readSlow();
void measure_slow(bool (*checkRadio)(void));
void reInitVars();
bool checkRadio();

//pins for arduino mega
namespace pin {
	constexpr int RADIO_CE = 48;
	constexpr int RADIO_CS = 49;
}

namespace NODE_CENTRAL{
	constexpr uint8_t addr[] = "0No";
}

constexpr uint8_t PIPE = 1;

namespace NODE_BED{
	constexpr uint8_t addr[] = "1No";
	constexpr uint8_t LEN_fBuf = 10;
	constexpr uint8_t LEN_sBuf = 10;

	uint8_t sBuf[LEN_sBuf];
}

namespace headers{
	constexpr uint8_t RQ_FAST = 0;
	constexpr uint8_t RQ_MEASURE_SLOW = 1;
	constexpr uint8_t RQ_READ_SLOW = 2;
	constexpr uint8_t RQ_INIT = 3;

	constexpr uint8_t SLOW_RDY = 1;
}

uint8_t addresses[][6] = {"1Node","2Node"}; //FIXME

RF24 radio(pin::RADIO_CE, pin::RADIO_CS);
bool reInit = false;
bool slowRdy = false;
uint8_t status = 0;

void setup(){ 
  Serial.begin(115200); //Open serial connection to report values to host
	printf_begin();

  radio.begin();
  //radio.setAddressWidth(3);          //sets adress with to 3 bytes long
  //radio.setAutoAck(1);               // Ensure autoACK is enabled
  //radio.setPayloadSize(5);                

  //radio.setRetries(15,15);            // Smallest time between retries, max no. of retries
	radio.setPALevel(RF24_PA_MIN);	  
  //radio.setDataRate(RF24_250KBPS);
	radio.setChannel(108);	            // 2.508 Ghz - Above most Wifi Channels

	radio.openWritingPipe(addresses[0]);//NODE_CENTRAL::addr);	
	radio.openReadingPipe(PIPE, addresses[1]);//NODE_BED::addr);	


  //radio.openWritingPipe(addresses[0]);
  //radio.openReadingPipe(1,addresses[1]);

	radio.startListening();            // Start listening  

	while(1){ //loop
		unsigned long got_time;
		if( radio.available()){
      while (radio.available()) radio.read( &got_time, sizeof(unsigned long) ); 
		  radio.stopListening();
		  radio.write( &got_time, sizeof(unsigned long) ); 
		  radio.startListening();
		  Serial.print(F("Sent response "));
		  Serial.println(got_time);
		}
	}
}


void reInitVars(){
	status = 0;
	reInit = true;
	slowRdy = false;
}

bool checkRadio(){
	uint8_t header;
	if(radio.available()){
		Serial.println("gotRadio");
		radio.read(&header, 1);
		Serial.print("gotheader: ");
		Serial.println("header\n");
		switch(header){
			case headers::RQ_FAST:
			handle_fast();
			break;
			case headers::RQ_READ_SLOW:
			handle_readSlow();
			break;
			case headers::RQ_INIT:
			Serial.println("init request recieved");
			reInitVars();
			break;

			case headers::RQ_MEASURE_SLOW:
			return true;
			break;
		}
	}
	return false;
}

void loop(){

	bool measureSlow = checkRadio();
	if(measureSlow) measure_slow(checkRadio);
}

void handle_fast(){
	uint8_t fBuf[NODE_BED::LEN_fBuf+1];
	fBuf[0] = status;

	//delay(10); //TODO simulates sensor reading taking time
	Serial.println("read fast sensors\n");
	radio.startListening();
	radio.write(fBuf, NODE_BED::LEN_fBuf+1);
	radio.startListening();
}

void handle_readSlow(){
	//no header in slow package
	uint8_t sBuf[NODE_BED::LEN_sBuf];

	Serial.println("sending slow data\n");
	radio.startListening();
	radio.write(sBuf, NODE_BED::LEN_fBuf+1);
	radio.startListening();
}

void measure_slow(bool (*checkRadio)(void)){
	Serial.println("reading continues sensors with registers");
	Serial.println("sending measure requests to other sensors");

	for(int i = 0; i<10; i++){
		//delay(10);
		Serial.println("polling if all sensors are ready for readout");
		checkRadio();
		if(reInit) return;
	}

	Serial.println("setting status to slow ready");
	status = headers::SLOW_RDY;
}
