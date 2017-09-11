#include <SPI.h>
#include "RF24.h"
#include <printf.h>
#include "fastSensors.h"
#include "encodingScheme.h"

namespace NODE_BATHROOM{
	constexpr uint8_t addr[] = "4Node"; //addr may only diff in first byte
	constexpr uint8_t LEN_fBuf = EncFastArduino::LEN_BATHROOM_NODE;
	constexpr uint8_t LEN_sBuf = EncSlowArduino::LEN_BATHROOM_NODE;
	uint8_t sBuf[LEN_sBuf];
}

//
void handle_fast();
void handle_readSlow();
void measure_slow(bool (*checkRadio)(void));
void reInitVars();
bool checkRadio(bool &measureSlow);



RF24 radio(pin::RADIO_CE, pin::RADIO_CS);
bool reInit = false;
bool slowRdy = false;
uint8_t slowMeasurementStatus = 0;

void setup(){ 
  Serial.begin(115200); //Open serial connection to report values to host
	configure_fast();
	printf_begin();

  radio.begin();
  //radio.setAutoAck(true);               // Ensure autoACK is enabled
  //radio.setPayloadSize(5);                

  radio.setRetries(1,5);            // Smallest time between retries, max no. of retries
	radio.setPALevel(RF24_PA_LOW);	  
  radio.setDataRate(RF24_250KBPS);
	radio.setChannel(108);	            // 2.508 Ghz - Above most Wifi Channels

	radio.openWritingPipe(NODE_CENTRAL::addr);	
	radio.openReadingPipe(PIPE, NODE_BATHROOM::addr);	

	radio.printDetails();
	radio.startListening();            // Start listening  
	Serial.println("Setup done\n");
}

void(* resetFunc) (void) = 0; //declare reset function @ address 0

void reInitVars(){
	Serial.println("re-initting vars\n");
	slowMeasurementStatus = 0;
	reInit = true;
	slowRdy = false;

	radio.stopListening();
	while(!radio.write(&headers::INIT_DONE, 1)){ }
	radio.startListening();	
}

bool checkRadio(bool &measureSlow){
	uint8_t header;
	if(radio.available()){
		radio.read(&header, 1);
		//Serial.print("gotheader: ");
		//Serial.println(header);
		switch(header){
			case headers::RQ_FAST:
			handle_fast();
			break;
			case headers::RQ_READ_SLOW:
			handle_readSlow();
			break;
			case headers::RQ_INIT:
			reInitVars();
			return true;
			break;
			case headers::RQ_MEASURE_SLOW:
			measureSlow = true;
			break;
		}
	}
	return false;
}

void loop(){
	uint8_t fBuf[NODE_BATHROOM::LEN_fBuf];
	bool measureSlow = false;

	checkRadio(measureSlow);
	if(measureSlow) measure_slow(checkRadio);
	readAndEncode(fBuf);
	//delay(5000);
}

void handle_fast(){
	uint8_t fBuf[NODE_BATHROOM::LEN_fBuf];
	memset(fBuf, 0, NODE_BATHROOM::LEN_fBuf);

	fBuf[0] = slowMeasurementStatus; //if moved after read and encode this works....

	readAndEncode(fBuf);

	radio.stopListening();
	radio.write(fBuf, NODE_BATHROOM::LEN_fBuf);
	radio.startListening();
}

void handle_readSlow(){
	//no header in slow package
	Serial.println("trying to send slowdata\n");
	radio.stopListening();
	if(radio.write(NODE_BATHROOM::sBuf, NODE_BATHROOM::LEN_sBuf))
		slowMeasurementStatus = 0; //reset slowMeasurementStatus only if slow deliverd succesfully
	radio.startListening();
}

void measure_slow(bool (*checkRadio)(void)){
	float tempC;
	memset(NODE_BATHROOM::sBuf, 0, NODE_BATHROOM::LEN_sBuf);

/*	Serial.println("sending measure requests to slow non continues sensors");*/
/*	//send request for data to sensors*/
/*	TempHumid::requestTemp();*/

/*	while(!TempHumid::readyToRead()){*/
/*		if(checkRadio()){return; }*/
/*	}*/
/*	tempC = TempHumid::readTemperatureC();*/
/*	Serial.println(tempC);*/
/*	encode(NODE_BATHROOM::sBuf, (uint16_t)((tempC*10) +100),*/
/*	       EncSlowArduino::TEMP_DOOR, EncSlowArduino::LEN_TEMP);*/
/*	TempHumid::requestHumid();*/

/*	while(!TempHumid::readyToRead()){*/
/*		if(checkRadio()) return;*/
/*	}*/
/*	encode(NODE_BATHROOM::sBuf, (uint16_t)(TempHumid::readHumidity(tempC)*10), */
/*	       EncSlowArduino::HUM_DOOR, EncSlowArduino::LEN_HUM);*/
	slowMeasurementStatus = headers::SLOW_RDY;
}
