/*
 *  This file is part of plainRFM69.
 *  Copyright (c) 2014, Ivor Wanders
 *  MIT License, see the LICENSE.md file in the root folder.
*/
#include <iostream>
#include "bareRFM69.h"
#define MICROSLEEP_LENGTH 15
// Most functions are implemented in the header file.

void bareRFM69::writeRegister(uint8_t reg, uint8_t data){
		//printf("%x %x\n", addr, value);
		uint8_t rawDATA[2];
		rawDATA[0] = reg | RFM69_WRITE_REG_MASK;
		rawDATA[1] = data;

		//spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, sizeof(rawDATA) );
		spiWrite(spi_handle, (char*)rawDATA, sizeof(rawDATA) );
}

uint8_t bareRFM69::readRegister(uint8_t reg){
    uint8_t foo[] = {0,0};
		foo[0] = reg & RFM69_READ_REG_MASK; 
		
		spiXfer(spi_handle, (char*)foo, (char*)foo, sizeof(foo) );
    return foo[1];
}

/*void bareRFM69::writeMultiple(uint8_t reg, void* data, uint8_t len){
		reg = RFM69_WRITE_REG_MASK | (reg & RFM69_READ_REG_MASK);
    spiWrite(spi_handle, (char*)&reg, 1); 
    char* r = reinterpret_cast<char*>(data);
    for (uint8_t i=0; i < len ; i++){
        spiWrite(spi_handle, &r[len - i - 1], 1);
    }
}*/
void bareRFM69::writeMultiple(uint8_t reg, void* data, uint8_t len){
		char* buf = new char[len+1]; 
		buf[0] = reg | RFM69_WRITE_REG_MASK;
		for(int i=0, j=len; i<len; i++, j--)
			buf[j] = ((char*)data)[i];
		
    spiXfer(spi_handle, buf, buf, len+1); 
    //spiWrite(spi_handle, &r[len - i - 1], &r[len - i - 1], 1);
}
/*
void bareRFM69::readMultiple(uint8_t reg, void* data, uint8_t len){   
		reg = reg % RFM69_READ_REG_MASK;
    spiWrite(spi_handle, (char*)&reg, 1); 
    char* r = reinterpret_cast<char*>(data);
    for (uint8_t i=0; i < len ; i++){
        spiRead(spi_handle, &r[len - i - 1], 1);
    }
}*/
void bareRFM69::readMultiple(uint8_t reg, void* data, uint8_t len){   
		char* buf = new char[len+1]; 
		buf[0] = reg & RFM69_READ_REG_MASK; 
		memcpy(buf+1, data, len);
    spiXfer(spi_handle, buf, buf, len+1);	
		for(int i=0, j=len; i<len; i++, j--)
			((char*)data)[i] = buf[j];
}

uint32_t bareRFM69::readRegister32(uint8_t reg){
    uint32_t f = 0;
    this->readMultiple(reg, &f, 4);
    return f;
}
uint32_t bareRFM69::readRegister24(uint8_t reg){
    uint32_t f = 0;
    this->readMultiple(reg, &f, 3);
    return f;
}
uint16_t bareRFM69::readRegister16(uint8_t reg){
    uint16_t f = 0;
    this->readMultiple(reg, &f, 2);
    return f;
}

void bareRFM69::writeFIFO(void* buffer, uint8_t len){
    char* r = reinterpret_cast<char*>(buffer);
		char reg = RFM69_WRITE_REG_MASK | (RFM69_FIFO & RFM69_READ_REG_MASK);
    spiWrite(spi_handle, &reg, 1); 
		spiWrite(spi_handle, r, len);
}

void bareRFM69::readFIFO(void* buffer, uint8_t len){
    char* r = reinterpret_cast<char*>(buffer);
		char reg = RFM69_FIFO % RFM69_READ_REG_MASK;
		spiWrite(spi_handle, &reg, 1); 
		spiRead(spi_handle, r, len);
}

uint8_t bareRFM69::readVariableFIFO(void* buffer, uint8_t max_length){
    char* r = reinterpret_cast<char*>(buffer);
    
    spiWrite(spi_handle, RFM69_FIFO % RFM69_READ_REG_MASK, 1); 
    uint8_t len; spiRead(spi_handle, (char*)&len, 1);
    r[0] = len;
    // Serial.print("readVariableFIFO, len:"); Serial.println(len);
    len = len > (max_length-1) ? (max_length-1) : len;
    // Serial.print("readVariableFIFO, len:"); Serial.println(len);
		spiRead(spi_handle, r+1, len);
     // Serial.print("readVariableFIFO, r[i+1]"); Serial.println(r[i+1]);
    return len;
}

 void bareRFM69::reset(uint8_t pin){ // function to send the RFM69 a hardware reset.
    // p 75 of datasheet;
    //pinMode(pin, OUTPUT);
    //digitalWrite(pin, HIGH);
    //delayMicroseconds(150); // pull high for >100 uSec
    //pinMode(pin, INPUT); // release
    //delay(10); //  wait 10 milliseconds before SPI is possible.
}

uint32_t timeMicroSec(){
	timeval tv;	
	gettimeofday(&tv, nullptr);
	return tv.tv_usec;
}

uint32_t millis(){
	return timeMicroSec()*1000;
}

void delayMicroseconds(int dt){
	std::this_thread::sleep_for (std::chrono::microseconds(dt));
}