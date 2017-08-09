#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include <printf.h>
#include "fastSensors.h"
#include "humiditySensor.h"
#include "pressure.h"
#include "co2.h"
#include "encodingScheme.h"

namespace NODE_BED{
	constexpr uint8_t addr[] = "2Node"; //addr may only diff in first byte
	constexpr uint8_t LEN_fBuf = EncFastArduino::LEN_BED_NODE;
	constexpr uint8_t LEN_sBuf = EncSlowArduino::LEN_BED_NODE;
	uint8_t sBuf[LEN_sBuf];
}

//
void handle_fast();
void handle_readSlow();
void measure_slow(bool (*checkRadio)(void));
void reInitVars();
bool checkRadio();



RF24 radio(pin::RADIO_CE, pin::RADIO_CS);
Adafruit_BMP280 pressure;
bool reInit = false;
bool slowRdy = false;
uint8_t slowMeasurementStatus = 0;

void setup(){ 
  Serial.begin(115200); //Open serial connection to report values to host
	printf_begin();

  radio.begin();
  //radio.setAutoAck(true);               // Ensure autoACK is enabled
  //radio.setPayloadSize(5);                

  radio.setRetries(1,5);            // Smallest time between retries, max no. of retries
	radio.setPALevel(RF24_PA_MIN);	  
  radio.setDataRate(RF24_250KBPS);
	radio.setChannel(108);	            // 2.508 Ghz - Above most Wifi Channels

	radio.openWritingPipe(NODE_CENTRAL::addr);	
	radio.openReadingPipe(PIPE, NODE_BED::addr);	

	radio.printDetails();
	radio.startListening();            // Start listening  

	//setup sensors
	Co2::setup();
	pressure.setup();
}


void reInitVars(){
	Serial.println("re-initting vars\n");
	slowMeasurementStatus = 0;
	reInit = true;
	slowRdy = false;
	radio.stopListening();
	radio.write(&headers::INIT_DONE, 1);
	radio.startListening();	
}

bool checkRadio(){
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
	uint8_t fBuf[NODE_BED::LEN_fBuf];
	memset(fBuf, 0, NODE_BED::LEN_fBuf);

	fBuf[0] = slowMeasurementStatus; //if moved after read and encode this works....

	readAndEncode(fBuf);

	radio.stopListening();
	radio.write(fBuf, NODE_BED::LEN_fBuf);
	radio.startListening();
}

void handle_readSlow(){
	//no header in slow package
	Serial.println("trying to send slowdata\n");
	radio.stopListening();
	if(radio.write(NODE_BED::sBuf, NODE_BED::LEN_sBuf))
		slowMeasurementStatus = 0; //reset slowMeasurementStatus only if slow deliverd succesfully
	radio.startListening();
}

void measure_slow(bool (*checkRadio)(void)){
	float tempC;
	memset(NODE_BED::sBuf, 0, NODE_BED::LEN_sBuf);

	//Serial.println("sending measure requests to slow non continues sensors");
	//send request for data to sensors
	Co2::request();
	TempHumid::requestTemp();

	//Serial.println("reading continues sensors with registers");
	//reading continues sensors with registers
	encode(NODE_BED::sBuf, pressure.readPressure(), 
	       EncSlowArduino::PRESSURE, EncSlowArduino::LEN_PRESSURE);

	while(!reInit && !TempHumid::readyToRead()){
		checkRadio();
		//Serial.println("polling if temp is ready for readout");
	}
	tempC = TempHumid::readTemperatureC();
	encode(NODE_BED::sBuf, (uint16_t)((tempC*10) +100),
	       EncSlowArduino::TEMP_BED, EncSlowArduino::LEN_TEMP);
	TempHumid::requestTemp();

	while(!reInit && !TempHumid::readyToRead()){
		checkRadio();
		//Serial.println("polling if humidity is ready for readout");
	}
	encode(NODE_BED::sBuf, (uint16_t)(TempHumid::readHumidity(tempC)*10), 
	       EncSlowArduino::HUM_BED, EncSlowArduino::LEN_HUM);

	while(!reInit && !Co2::readyToRead()){
		checkRadio();
		//Serial.println("polling if co2 is ready for readout");
	}
	encode(NODE_BED::sBuf, Co2::readCO2(), EncSlowArduino::CO2, 
	       EncSlowArduino::LEN_CO2);

	Serial.println("setting slowMeasurementStatus to slow ready");
	slowMeasurementStatus = headers::SLOW_RDY;
}
