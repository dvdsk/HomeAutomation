/*
 *  Copyright (c) 2014, Ivor Wanders
 *  MIT License, see the LICENSE.md file in the root folder.
*/

#include <SPI.h>
#include <plainRFM69.h>

// slave select pin.
#define SLAVE_SELECT_PIN 8     

// connected to the reset pin of the RFM69.
#define RESET_PIN 23



/*
    This is very minimal, it does not use the interrupt.

    Using the interrupt is recommended.
*/

plainRFM69 rfm = plainRFM69(SLAVE_SELECT_PIN);

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
            Serial.print("Send:");Serial.println(counter);

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
            Serial.print("Packet ("); Serial.print(len); Serial.print("): "); Serial.println(received_count);

            if (counter+1 != received_count){
                // if the increment is larger than one, we lost one or more packets.
                Serial.println("Packetloss detected!");
            }

            // assign the received counter to our counter.
            counter = received_count;
        }
    }
}

void setup(){
    Serial.begin(9600);
    SPI.begin();

    bareRFM69::reset(RESET_PIN); // sent the RFM69 a hard-reset.

    rfm.setRecommended(); // set recommended paramters in RFM69.
    rfm.setPacketType(false, false); // set the used packet type.

    rfm.setBufferSize(2);   // set the internal buffer size.
    rfm.setPacketLength(4); // set the packet length.
    rfm.setNodeAddress(98);
    
    rfm.setFrequency((uint32_t) 434*1000*1000); // set the frequency.
    Serial.println("setup done!");
    // baudrate is default, 4800 bps now.
    rfm.baud4800();
    rfm.receive();
    // set it to receiving mode.

    delay(5);
}

void loop(){
    receiver(); 
    //while(1){
    //  rfm.startRssi();
    //  while(!rfm.completedRssi());
    //  Serial.println(rfm.getRssiValue());
    //  delay(5);
  //}
}


