#include "radio.h"

RemoteNodes::RemoteNodes(uint16_t* fastData_, uint16_t* slowData_)
: RF24(pin::RADIO_CE, pin::RADIO_CS){

	fastData = fastData_;
	slowData = slowData_;

	//initialise and configure radio
  RF24::begin();
  RF24::setAddressWidth(3);               //sets adress with to 3 bytes long
  RF24::setAutoAck(1);                    // Ensure autoACK is enabled
  RF24::enableAckPayload();               // Allow optional ack payloads
  RF24::setPayloadSize(5);                

  RF24::setRetries(0,15);                  // Smallest time between retries, max no. of retries
	RF24::setPALevel(RF24_PA_MIN);	  
  //radio.setDataRate(RF24_250KBPS);
  RF24::setChannel(108);// 2.508 Ghz - Above most Wifi Channels
	
	radio.startListening();                 // Start listening  
	#ifdef DEBUG
  RF24::printDetails();                   // Dump the configuration of the rf unit for debugging
	#endif
	RF24::stopListening();

}

void RemoteNodes::requestSlowUpdate(){
	currentRq_N1 = radioRQ::NODE1_SLOW_UPDATE;
	currentRq_N2 = radioRQ::NODE2_SLOW_UPDATE;
}

void RemoteNodes::pollNodes(){
	poll_N1();
	poll_N2();
}

void RemoteNodes::poll_N1(){
	uint8_t rcbuffer[5];

  RF24::openWritingPipe(RADIO_ADDRESSES[1]);
	Serial.print("Hello1");
  if(RF24::write(&currentRq_N1, 1)){; //write currentRq to the currently opend writingPipe     
		if(RF24::available() ){    
			RF24::read(&rcbuffer, 5);//empty internal buffer from awk package	
		
			if(rcbuffer[0] != headers::FAST_UPDATE && outstandingSlowUpdate_N1){
			//SlowUpdate package recieved and we were waiting for a slow package.
				currentRq_N1 = radioRQ::NODE1_FAST_UPDATE;
				handleSlowData_N1();
			}
			else{
				handleFastData_N1();
			}
		}	
	}
	Serial.print("Hello2");	
}

void RemoteNodes::poll_N2(){
	uint8_t rcbuffer[5];

  RF24::openWritingPipe(RADIO_ADDRESSES[2]);
  RF24::write(&currentRq_N2, 1); //write currentRq to the currently opend writingPipe  
	Serial.print("Hello2");  
	if(RF24::available() ){
    RF24::read(&rcbuffer, 5);//empty internal buffer from awk package	
		
		if(rcbuffer[0] != headers::FAST_UPDATE && outstandingSlowUpdate_N2){
		//SlowUpdate package recieved and we were waiting for a slow package.
			currentRq_N2 = radioRQ::NODE2_FAST_UPDATE;
			handleSlowData_N2();
		}
		else{
			handleFastData_N2();
		}
	}
}

void RemoteNodes::handleSlowData_N1(){}
void RemoteNodes::handleSlowData_N2(){}

void RemoteNodes::handleFastData_N1(){}
void RemoteNodes::handleFastData_N2(){}
