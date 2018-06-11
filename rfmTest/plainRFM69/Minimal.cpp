
#include "RFM69HubNetwork.h"
#include <iostream>

RFM69HubNetwork sensorNet("test", 99, 434*1000*1000);
/*
plainRFM69 rfm = plainRFM69();

void sender(){

    uint32_t start_time = millis();

    uint32_t counter = 0; // the counter which we are going to send.
	uint8_t test[16] = {1};
    while(true){
        rfm.poll(); // run poll as often as possible.

        if (!rfm.canSend()){
            continue; // sending is not possible, already sending.
        }

        if ((millis() - start_time) > 500){ // every 500 ms. 
            start_time = millis();

            // be a little bit verbose.
            std::cout<<"Send:"<<counter<<std::endl;

            // send the number of bytes equal to that set with setPacketLength.
            // read those bytes from memory where counter starts.
            //rfm.sendAddressed(98, &counter);
            //rfm.sendAddressedVariable(98, &counter, 4);
			rfm.sendAddressedVariable(98, test, 16);
			
            counter++; // increase the counter.
        }
       
    }
}
*/
int main(){
	sensorNet.init();
	sensorNet.baud9600();
	/*
	uint8_t buffer[16] = {1};
	uint8_t command = 1, address = 98;
	uint32_t timeOutInBetween = 15;
	
	uint32_t start_time = millis();
	int i=0;
    while( i<10){
		if(sensorNet.SendCommandUntilAnswered_withTimeout(command, address, buffer, timeOutInBetween, 10)){
			i++;
			std::cout<<"---"<<std::endl;
		} else{}

	}
	std::cout<<(uint32_t)(millis() - start_time)<<std::endl;
	*/
	
	/*
	uint32_t start_time = millis();
    uint32_t counter = 0; // the counter which we are going to send.
	uint8_t test[16] = {1};
    while(true){
        sensorNet.poll(); // run poll as often as possible.

        if (!sensorNet.canSend()){
            continue; // sending is not possible, already sending.
        } else if ((millis() - start_time) > 1){ // every 500 ms. 
            start_time = millis();

            // be a little bit verbose.
            std::cout<<"Send:"<<counter<<std::endl;

            // send the number of bytes equal to that set with setPacketLength.
            // read those bytes from memory where counter starts.
            //rfm.sendAddressed(98, &counter);
            //rfm.sendAddressedVariable(98, &counter, 4);
			sensorNet.sendAddressedVariable(98, test, 16);
			
            counter++; // increase the counter.
        }
       
    }
	*/
	
	///*	
	uint32_t start_time = millis();
    uint32_t counter = 0; // the counter which we are going to send.
	uint8_t test[16] = {1};
	int i=0;
    while( i<100){
        sensorNet.poll(); // run poll as often as possible.

        if (!sensorNet.canSend()){
            continue; // sending is not possible, already sending.
        } else if ((millis() - start_time) > 1){ // every 500 ms. 
            start_time = millis();

            // be a little bit verbose.
            std::cout<<"Send:"<<counter<<std::endl;

            // send the number of bytes equal to that set with setPacketLength.
            // read those bytes from memory where counter starts.
            //rfm.sendAddressed(98, &counter);
            //rfm.sendAddressedVariable(98, &counter, 4);
			sensorNet.sendAddressedVariable(98, test, 16);
			i++;
			
            counter++; // increase the counter.
        }
       
    }
	//*/
}






/*


#include "plainRFM69.h"
#include "bareRFM69.h"
#include <iostream>
#include <bitset>

// slave select pin.
#define SLAVE_SELECT_PIN 10     

// connected to the reset pin of the RFM69.
#define RESET_PIN 23

// tie this pin down on the receiver.
#define SENDER_DETECT_PIN 15


#define KEY "sampleEncryptKey"

plainRFM69 rfm = plainRFM69();

void sender(){

    uint32_t start_time = millis();

    uint32_t counter = 0; // the counter which we are going to send.

    while(true){
        rfm.poll(); // run poll as often as possible.

        if (!rfm.canSend()){
            continue; // sending is not possible, already sending.
        }

        if ((millis() - start_time) > 500){ // every 500 ms. 
            start_time = millis();

            // be a little bit verbose.
            std::cout<<"Send:"<<counter<<std::endl;

            // send the number of bytes equal to that set with setPacketLength.
            // read those bytes from memory where counter starts.
            rfm.sendAddressed(98, &counter);
            
            counter++; // increase the counter.
        }
       
    }
}


int main(){
    bareRFM69::reset(RESET_PIN); // sent the RFM69 a hard-reset.

    rfm.setRecommended(); // set recommended paramters in RFM69.
	rfm.setAES(false);
		//rfm.bareRFM69::setAesKey((void*)KEY, (int)sizeof(KEY));
    rfm.setPacketType(false, false); // set the used packet type.

    rfm.setBufferSize(2);   // set the internal buffer size.
    rfm.setPacketLength(4); // set the packet length.
	rfm.setNodeAddress(99);
    
	rfm.setFrequency((uint32_t) 434*1000*1000); // set the frequency.
	rfm.setPALevel(RFM69_PA_LEVEL_PA0_ON, 31);
	rfm.baud9600();
    rfm.receive();	
	sender();
}
*/