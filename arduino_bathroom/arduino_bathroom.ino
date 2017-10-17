#include <SPI.h>
#include "RF24.h"
#include <printf.h>
#include "config.h"
#include "fastSensors.h"
#include "encodingScheme.h"
#include "libSHT31.h"
#ifdef DEBUG
	#include "ApplicationMonitor.h"
#endif


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

uint32_t start;

RF24 radio(pin::RADIO_CE, pin::RADIO_CS);
#ifdef DEBUG
	Watchdog::CApplicationMonitor ApplicationMonitor;
#endif

bool reInit = false;
bool slowRdy = false;
uint8_t slowMeasurementStatus = 0;

void setup(){ 
  Serial.begin(115200); //Open serial connection to report values to host
	db("in setup") 
	printf_begin();

  radio.begin();
  //radio.setAutoAck(true);               // Ensure autoACK is enabled
  //radio.setPayloadSize(5);                

  radio.setRetries(1,15);            // Smallest time between retries, max no. of retries
	radio.setPALevel(RF24_PA_HIGH);	  
  radio.setDataRate(RF24_250KBPS);
	radio.setChannel(108);	            // 2.508 Ghz - Above most Wifi Channels

	radio.openWritingPipe(NODE_CENTRAL::addr);	
	radio.openReadingPipe(PIPE, NODE_BATHROOM::addr);	

	radio.printDetails();
	radio.startListening();            // Start listening  

	//setup sensors
	configure_fast();
	TempHum::begin();

	#ifdef DEBUG
		Serial.println("Debuginfo: ");
		ApplicationMonitor.Dump(Serial);
		ApplicationMonitor.EnableWatchdog(Watchdog::CApplicationMonitor::Timeout_4s);
	#endif
}

void(* resetFunc) (void) = 0; //declare reset function @ address 0

void reInitVars(){
	db("in reInitVars") 
	slowMeasurementStatus = 0;
	reInit = true;
	slowRdy = false;
	TempHum::reset();

	radio.stopListening();
/*	unsigned long start = millis();*/
	radio.write(&headers::INIT_DONE, 1, 100);
/*	while(!radio.write(&headers::INIT_DONE, 1)){*/
/*		if ((unsigned long)(millis() - start) >= 100){*/
/*			Serial.println("re-init failed\n");*/
/*			radio.startListening();*/
/*			return;	//give up after 0.5 seconds*/
/*		}*/
/*	}*/
	radio.startListening();
	Serial.println("re-init complete\n");
}

//version for calling from readSlow funct,
//only two headers will behandled in this case
bool checkRadio(){
	db("in checkRadio") 
	#ifdef DEBUG
	ApplicationMonitor.IAmAlive();
	#endif

	uint8_t header;
	if(radio.available()){
		radio.read(&header, 1);
		switch(header){
			case headers::RQ_FAST:
				handle_fast();
				break;
			case headers::RQ_INIT:
				reInitVars();
				return true;
			break;
				default:
				break;
		}
	}
/*	//TODO FIXME enter an endless loop*/
/*	while(1); */
	return false;
}


bool checkRadio(bool &measureSlow){
	db("in checkRadio") 
	#ifdef DEBUG
	ApplicationMonitor.IAmAlive();
	#endif

	uint8_t header;
	if(radio.available()){
		radio.read(&header, 1);
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
	db("in loop") 
	bool measureSlow = false;

	checkRadio(measureSlow);
	if(measureSlow) measure_slow(checkRadio);
}

void handle_fast(){
	db("in handle_fast") 
	uint8_t fBuf[NODE_BATHROOM::LEN_fBuf];
	memset(fBuf, 0, NODE_BATHROOM::LEN_fBuf);

	fBuf[0] = slowMeasurementStatus; //if moved after read and encode this works....

	readAndEncode(fBuf);

	radio.stopListening();
	radio.write(fBuf, NODE_BATHROOM::LEN_fBuf);
	radio.startListening();
	db("out handle_fast") 
}

void handle_readSlow(){
	db("in handle_readSlow") 
	//no header in slow package
	radio.stopListening();
	if(radio.write(NODE_BATHROOM::sBuf, NODE_BATHROOM::LEN_sBuf))
		slowMeasurementStatus = 0; //reset slowMeasurementStatus only if slow deliverd succesfully
	radio.startListening();
}

void measure_slow(bool (*checkRadio)(void)){
	db("in measure_slow") 
	uint16_t temp, hum;

	TempHum::request();
	memset(NODE_BATHROOM::sBuf, 0, NODE_BATHROOM::LEN_sBuf);

	while(!TempHum::readyToRead()){
		if(checkRadio()){return; }
	}
	TempHum::read(temp, hum);
	encode(NODE_BATHROOM::sBuf, temp,
	       EncSlowArduino::TEMP_DOOR, EncSlowArduino::LEN_TEMP);
	encode(NODE_BATHROOM::sBuf, hum, 
	       EncSlowArduino::HUM_DOOR, EncSlowArduino::LEN_HUM);

	slowMeasurementStatus = headers::SLOW_RDY;
}
