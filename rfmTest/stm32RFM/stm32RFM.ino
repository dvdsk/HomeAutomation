/*
 *  Copyright (c) 2014, Ivor Wanders
 *  MIT License, see the LICENSE.md file in the root folder.
*/

#include <SPI.h>
#include "RFM69HubNetwork.h"
#include <Arduino.h>

// slave select pin.
#define SLAVE_SELECT_PIN PA4

// Pin DIO 2 on the RFM69 is attached to this digital pin.
// Pin should have interrupt capability.
#define DIO2_PIN PB1

// on Arduino UNO the the pin number is not what's given to attachInterrupt(..)
#define INTERRUPT_NUMBER 0
// On the Arduino UNO pin 2 has interrupt number 0.


//    Using the first SPI port (SPI_1)
//    SS    <-->  PA4 <-->  BOARD_SPI1_NSS_PIN
//    SCK   <-->  PA5 <-->  BOARD_SPI1_SCK_PIN
//    MISO  <-->  PA6 <-->  BOARD_SPI1_MISO_PIN
//    MOSI  <-->  PA7 <-->  BOARD_SPI1_MOSI_PIN


/*
    This is very minimal, it does not use the interrupt.

    Using the interrupt is recommended.
*/

RFM69HubNetwork sensorNet((WiringPinMode)SLAVE_SELECT_PIN, "test", 98, (uint32_t)434*1000*1000);

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
    //uint32_t counter = 0; // to count the messages.
    uint16_t counter = 0; // to count the messages.

    while(true){
        sensorNet.poll();
        while(sensorNet.available()){ // for all available messages:

            uint8_t received_count[17];
            *(uint32_t*)&received_count[1] = 0;
            uint8_t len = sensorNet.read(received_count); // read the packet into the new_counter.

            // print verbose output.
             Serial.print("Packet ("); Serial.print(counter); Serial.print("): "); Serial.println(*(uint32_t*)&received_count[1]);
             //Serial.print("Packet ("); Serial.print(len); Serial.print("): "); Serial.println(received_count[1]);
            counter++;
            //if (counter+1 != *(uint32_t*)&received_count[1]){
                // if the increment is larger than one, we lost one or more packets.
                //Serial.println("Packetloss detected!");
            //}

            // assign the received counter to our counter.
            //counter = *(uint32_t*)&received_count[1];
        }
    }
}

void interrupt_RFM(){
    sensorNet.poll(); // in the interrupt, call the poll function.
}

void setup(){
    Serial.begin(115200);
    while ( !Serial.isConnected() ) ; // wait till serial connection is setup, or serial monitor started
    delay(2000);
    SPI.begin();
    SPI.setBitOrder(MSBFIRST); // Set the SPI_1 bit order
    SPI.setDataMode(SPI_MODE0); //Set the  SPI_1 data mode 0
    SPI.setClockDivider(SPI_CLOCK_DIV8);      // Slow speed (72 / 16 = 4.5 MHz SPI_1 speed)
      
    sensorNet.init();
    sensorNet.baud9600();

    if(sensorNet.isConnected()) Serial.println("Radio is connected");
    
    // tell the RFM to represent whether we are in automode on DIO 2.
    sensorNet.setDioMapping1(RFM69_PACKET_DIO_2_AUTOMODE);

    // set pinmode to input.
    pinMode(DIO2_PIN, INPUT);

    // Tell the SPI library we're going to use the SPI bus from an interrupt.
    //SPI.usingInterrupt(DIO2_PIN);

    // hook our interrupt function to any edge.
    attachInterrupt(DIO2_PIN, interrupt_RFM, CHANGE);

    // start receiving.
    sensorNet.receive();
    Serial.print("setup done");
    delay(5);
}

void loop(){
    receiver(); 
}


