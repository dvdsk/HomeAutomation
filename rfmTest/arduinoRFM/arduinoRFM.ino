/*
 *  Copyright (c) 2014, Ivor Wanders
 *  MIT License, see the LICENSE.md file in the root folder.
*/

#include <SPI.h>
#include "RFM69HubNetwork.h"

// slave select pin.
#define SLAVE_SELECT_PIN 8

// Pin DIO 2 on the RFM69 is attached to this digital pin.
// Pin should have interrupt capability.
#define DIO2_PIN 2

// on Arduino UNO the the pin number is not what's given to attachInterrupt(..)
#define INTERRUPT_NUMBER 0
// On the Arduino UNO pin 2 has interrupt number 0.



/*
    This is very minimal, it does not use the interrupt.

    Using the interrupt is recommended.
*/

RFM69HubNetwork sensorNet(SLAVE_SELECT_PIN, "test", 98, (uint32_t)434*1000*1000);

void sender(){

    uint32_t start_time = millis();

    uint32_t counter = 0; // the counter which we are going to send.

    while(true){
        //rfm.poll(); // run poll as often as possible.

        if (!sensorNet.canSend()){
            continue; // sending is not possible, already sending.
        }

        if ((millis() - start_time) > 500){ // every 500 ms. 
            start_time = millis();

            // be a little bit verbose.
            Serial.print("Send:");Serial.println(counter);

            // send the number of bytes equal to that set with setPacketLength.
            // read those bytes from memory where counter starts.
            sensorNet.send(&counter);
            
            counter++; // increase the counter.
        }
       
    }
}

void receiver(){
    uint32_t counter = 0; // to count the messages.

    while(true){
        while(sensorNet.available()){ // for all available messages:

            uint8_t received_count[17];
            *(uint32_t*)&received_count[1] = 0;
            uint8_t len = sensorNet.read(received_count); // read the packet into the new_counter.

            // print verbose output.
              Serial.print("Packet ("); Serial.print(len); Serial.print("): "); Serial.println(*(uint32_t*)&received_count[1]);
             //Serial.print("Packet ("); Serial.print(len); Serial.print("): "); Serial.println(received_count[1]);

            if (counter+1 != received_count){
                // if the increment is larger than one, we lost one or more packets.
                //Serial.println("Packetloss detected!");
            }

            // assign the received counter to our counter.
            counter = received_count;
        }
    }
}

void interrupt_RFM(){
    Serial.println("ttttt");
    sensorNet.poll(); // in the interrupt, call the poll function.
}

void setup(){
    Serial.begin(115200);
    delay(2000);
    SPI.begin();
    
    sensorNet.init();
    sensorNet.baud9600();
    
    // tell the RFM to represent whether we are in automode on DIO 2.
    sensorNet.setDioMapping1(RFM69_PACKET_DIO_2_AUTOMODE);

    // set pinmode to input.
    pinMode(DIO2_PIN, INPUT);

    // Tell the SPI library we're going to use the SPI bus from an interrupt.
    SPI.usingInterrupt(INTERRUPT_NUMBER);

    // hook our interrupt function to any edge.
    attachInterrupt(INTERRUPT_NUMBER, interrupt_RFM, CHANGE);

    // start receiving.
    sensorNet.receive();
    Serial.print("setup done");
    delay(5);
}

void loop(){
    receiver(); 
}


