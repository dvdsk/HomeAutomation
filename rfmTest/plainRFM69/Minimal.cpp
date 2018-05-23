/*
 *  Copyright (c) 2014, Ivor Wanders
 *  MIT License, see the LICENSE.md file in the root folder.
*/

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

/*
    This is very minimal, it does not use the interrupt.

    Using the interrupt is recommended.
*/
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
            rfm.send(&counter);
            
            counter++; // increase the counter.
        }
       
    }
}

void receiver(){
	uint32_t counter = 0; // to count the messages.

	while(true){

		rfm.poll(); // poll as often as possible.

		while(rfm.available()){ // for all available messages:
			uint32_t received_count = 0; // temporary for the new counter.
			uint8_t len = rfm.read(&received_count); // read the packet into the new_counter.

			// print verbose output.
			std::cout<<"Packet ("<<len<<"): "<<received_count<<std::endl;
			//Serial.print("Packet ("); Serial.print(len); Serial.print("): "); Serial.println(received_count);

			if (counter+1 != received_count){
				// if the increment is larger than one, we lost one or more packets.
				std::cout<<"Packetloss detected!"<<std::endl;
			}

			// assign the received counter to our counter.
			counter = received_count;
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
		//rfm.setFrf(433*1000*1000/61.03515625);
		//rfm.baud9600(); // set the modulation parameters.
	rfm.baud4800();
    rfm.receive();
	//sender();	
	//while(1){
	//	rfm.startRssi();
	//	while(!rfm.completedRssi());
	//	std::cout<<rfm.getRssiValue()<<std::endl;
	//	delayMicroseconds(1000000);
	//}
	
	sender();
    	/*	
		uint32_t freqHz = 433*1000*1000;
		freqHz /= 61; // divide down by FSTEP to get FRF
		rfm.setFrequency((uint32_t) 433*1000*1000); // set the frequency.
		std::cout<<freqHz<<"\t\t"<<std::bitset<32>(freqHz)<<std::endl;
		rfm.readMultiple(RFM69_FRF_MSB, &freqHz, 3);
		std::cout<<freqHz<<"\t\t"<<std::bitset<32>(freqHz)<<std::endl;
		std::cout<<rfm.readRegister24(RFM69_FRF_MSB)*61<<std::endl;

		#define REG_FRFLSB        0x09
		rfm.writeRegister(REG_FRFLSB, 22);
		std::cout<<+rfm.readRegister(REG_FRFLSB)<<std::endl;

		const char* test = "ELLO WORLD";
		char testres[10];
		rfm.writeFIFO((void*)test, sizeof(test));
		rfm.readFIFO((void*)testres, sizeof(test));
		for(unsigned int i; i<sizeof(test); i++)
			std::cout<<testres[i];
		std::cout<<std::endl;
		//receiver();
		*/
    // set it to receiving mode.
		
}


