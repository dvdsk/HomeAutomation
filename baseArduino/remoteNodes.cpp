#include "remoteNodes.h"

void RemoteNodes::setup(uint16_t* fastData_, uint16_t* slowData_, RF24* radio_){

	radio = radio_;
	fastData = fastData_;
	slowData = slowData_;

	currentRq_N1 = radioRQ::NODE1_FAST_UPDATE;
	currentRq_N1 = radioRQ::NODE2_FAST_UPDATE;

	//initialise and configure radio
  radio->begin();
  radio->setAddressWidth(3);               //sets adress with to 3 bytes long
  radio->setAutoAck(1);                    // Ensure autoACK is enabled
  radio->enableAckPayload();               // Allow optional ack payloads
  radio->setPayloadSize(5);                

  radio->setRetries(0,15);                  // Smallest time between retries, max no. of retries
	radio->setPALevel(RF24_PA_MIN);	  
  //radio.setDataRate(RF24_250KBPS);
  radio->setChannel(108);// 2.508 Ghz - Above most Wifi Channels
	
	radio->startListening();                 // Start listening  
	#ifdef DEBUG
  radio->printDetails();                   // Dump the configuration of the rf unit for debugging
	#endif
	radio->stopListening();
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

  radio->openWritingPipe(RADIO_ADDRESSES[1]);
  if(radio->write(&currentRq_N1, 1)){; //write currentRq to the currently opend writingPipe     
		if(radio->available() ){    
			radio->read(&rcbuffer, 5);//empty internal buffer from awk package	
		
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
}

void RemoteNodes::poll_N2(){
	uint8_t rcbuffer[5];

  radio->openWritingPipe(RADIO_ADDRESSES[2]);
  radio->write(&currentRq_N2, 1); //write currentRq to the currently opend writingPipe  
	if(radio->available() ){
    radio->read(&rcbuffer, 5);//empty internal buffer from awk package	
		
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
