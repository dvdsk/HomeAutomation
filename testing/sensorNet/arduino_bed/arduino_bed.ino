#include <Wire.h>
#include <SPI.h>
#include "RF24.h"
#include <printf.h>
#include "fastSensors.h"
#include "slowSensors.h"
#include "encodingScheme.h"


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
	constexpr uint8_t addr[] = "1Node"; //addr may only diff in first byte
}

constexpr uint8_t PIPE = 1;

namespace NODE_BED{
	constexpr uint8_t addr[] = "2Node"; //addr may only diff in first byte
	constexpr uint8_t LEN_fBuf = EncFastArduino::LEN_BEDNODE;
	constexpr uint8_t LEN_sBuf = EncSlowArduino::LEN_BEDNODE;
}

namespace headers{
	constexpr uint8_t RQ_FAST = 0;
	constexpr uint8_t RQ_MEASURE_SLOW = 1;
	constexpr uint8_t RQ_READ_SLOW = 2;
	constexpr uint8_t RQ_INIT = 3;

	constexpr uint8_t SLOW_RDY = 0b00000001;
}

uint8_t addresses[][6] = {"1Node","2Node"}; //FIXME

RF24 radio(pin::RADIO_CE, pin::RADIO_CS);
Adafruit_BMP280 pressure();

bool reInit = false;
bool slowRdy = false;
uint8_t slowMeasurementStatus = 0;

void setup(){ 
  Serial.begin(115200); //Open serial connection to report values to host
	printf_begin();

  radio.begin();
  //radio.setAutoAck(true);               // Ensure autoACK is enabled
  //radio.setPayloadSize(5);                

  //radio.setRetries(15,15);            // Smallest time between retries, max no. of retries
	radio.setPALevel(RF24_PA_MIN);	  
  radio.setDataRate(RF24_250KBPS);
	radio.setChannel(108);	            // 2.508 Ghz - Above most Wifi Channels

	radio.openWritingPipe(NODE_CENTRAL::addr);	
	radio.openReadingPipe(PIPE, NODE_BED::addr);	

	radio.printDetails();
	radio.startListening();            // Start listening  

	//setup sensors
	Co2::setup();
}


void reInitVars(){
	Serial.println("re-initting vars\n");
	slowMeasurementStatus = 0;
	reInit = true;
	slowRdy = false;
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
	fBuf[0] = slowMeasurementStatus;

	readAndEncode(fBuf); //TODO check if delay needed
	radio.stopListening();
	radio.write(fBuf, NODE_BED::LEN_fBuf+1);
	radio.startListening();
}

void handle_readSlow(){
	//no header in slow package
	uint8_t sBuf[NODE_BED::LEN_sBuf];

	Serial.println("sending read command to slowSens\n");
		

	radio.stopListening();
	if(radio.write(sBuf, NODE_BED::LEN_fBuf+1))
		slowMeasurementStatus = 0; //reset slowMeasurementStatus only if slow deliverd succesfully
	radio.startListening();

}

void measure_slow(bool (*checkRadio)(void)){
	float tempC;

	Serial.println("reading continues sensors with registers");
	//reading continues sensors with registers
	encode(buffer, pressure.readPressure(), EncSlowArduino::PRESSURE
	       EncSlowArduino::LEN_PRESSURE);

	Serial.println("sending measure requests to other sensors");
	//send request for data to sensors
	Co2::request();
	TempHumid::requestTemp();

	while(!reInit && !TempHumid::readyToRead()){
		checkRadio();
		Serial.println("polling if all sensors are ready for readout");
	}
	tempC = readTemperatureC();
	encode(buffer, (uint16_t)((tempC*10) +100), EncSlowArduino::TEMP_BED, 
	       EncSlowArduino::LEN_TEMP)
	TempHumid::requestTemp();

	while(!reInit && !TempHumid::readyToRead()){
		checkRadio();
		Serial.println("polling if all sensors are ready for readout");
	}
	encode(buffer, (uint16_t)(readHumidity(tempC)*10), EncSlowArduino::HUM_BED, 
	       EncSlowArduino::LEN_HUM)

	while(!reInit && !Co2::readyToMeasure()){
		checkRadio();
		Serial.println("polling if all sensors are ready for readout");
	}
	encode(buffer, readCO2(tempC), EncSlowArduino::CO2, 
	       EncSlowArduino::LEN_CO2)


	Serial.println("setting slowMeasurementStatus to slow ready");
	slowMeasurementStatus = headers::SLOW_RDY;
}
