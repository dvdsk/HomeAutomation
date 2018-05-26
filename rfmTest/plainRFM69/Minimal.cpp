#include "RFM69HubNetwork.h"

int main(){
	RFM69HubNetwork RFM69HubNetwork("test", 99, 434*1000*1000);
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

inline void receiveOnce_withAwk(uint8_t* buffer){
	rfm.poll(); // poll as often as possible.

	if(rfm.available()){ // for all available messages:
		uint8_t len = rfm.read(buffer); // read the packet into the new_counter.
		
	}
}

void receiveWithAwk(){
	
}

bool sendWithAwk(){
	
}

bool reciever(){
    while(true){
		rfm.poll();
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


int main(){
    bareRFM69::reset(RESET_PIN); // sent the RFM69 a hard-reset.

    rfm.setRecommended(); // set recommended paramters in RFM69.
	rfm.setAES(false);
		//rfm.bareRFM69::setAesKey((void*)KEY, (int)sizeof(KEY));
    rfm.setPacketType(false, true); // set the used packet type.

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
