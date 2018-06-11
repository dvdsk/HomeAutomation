#include "RFM69HubNetwork.h"



//bypass compiler and inline funct (comp doesnt know context)
__attribute__((always_inline)) bool RFM69HubNetwork::receive_tryOnce_withAwk(uint8_t* buffer, uint8_t awkAddr){
	poll();
	bool result = available();
	if(result){
		read(buffer);
		sendAwk(awkAddr);
	}
	return result;
}

__attribute__((always_inline)) bool RFM69HubNetwork::receive_tryOnce(uint8_t* buffer){
	poll();
	bool result = available();
	if(result){
		read(buffer);
	}
	return result;
}

RFM69HubNetwork::RFM69HubNetwork(WiringPinMode _slaveSelectPin, const char* _encryptionKey, uint8_t _hubAddr, uint32_t _freq)
	: plainRFM69(_slaveSelectPin), encryptionKey(_encryptionKey), hubAddr(_hubAddr), freq(_freq){
}

void RFM69HubNetwork::init(){
	
	bareRFM69::reset(10);
  if(!isConnected()) Serial.println("Radio not connected");
	setRecommended();
	setAES(false);
	//setAesKey((void*)encryptionKey, (int)sizeof(encryptionKey));
	setPacketType(true, true);
	//AES is enabled, length below 16 results in zero padding
	//lengths shorter than 16 bytes not faster.
	setBufferSize(10);
	setPacketLength(RFM69_PACKAGE_LEN); //bytes (16)
	setNodeAddress(hubAddr);
	
	setFrequency(freq);
  setPALevel(RFM69_PA_LEVEL_PA0_ON, 31);
  
	receive();
}

/* void RFM69HubNetwork::receive_tryForever_withAwk(uint8_t* buffer, uint8_t awkAddr){
	uint32_t T_start = timeMicroSec();
	while(!receive_tryOnce_withAwk(buffer, awkAddr) );
} */

bool RFM69HubNetwork::tryReceiveWithTimeout_sendAwk(uint8_t* buffer, uint32_t timeOut, uint8_t awkAddr){
	uint32_t T_start = micros();
	bool result = receive_tryOnce_withAwk(buffer, awkAddr);
	while(!result)
		result = receive_tryOnce_withAwk(buffer, awkAddr) and ((uint32_t)(micros()-T_start) < timeOut);
	return result;	
}

bool RFM69HubNetwork::tryReceiveWithTimeout(uint8_t* buffer, uint32_t timeOut){
	uint32_t T_start = micros();
	bool result = receive_tryOnce(buffer);
	while(!result)
		result = receive_tryOnce(buffer) and ((uint32_t)(micros()-T_start) < timeOut);
	return result;	
}

//bool RFM69HubNetwork::

bool RFM69HubNetwork::SendCommandUntilAwknowledged_withTimeout(uint8_t command, uint8_t address, uint32_t timeOut){
	uint32_t T_start = micros();
	bool result = waitForAwk(10);
	while(!result){
		sendAddressedVariable(address, &command, 1);
		result = waitForAwk(10) and (uint32_t)(micros()-T_start) < timeOut;
	}
	return result;	
}

bool RFM69HubNetwork::SendCommandUntilAnswered_withTimeout(uint8_t command, uint8_t address, uint8_t* buffer, uint32_t timeOut){
	uint32_t T_start = micros();
	bool result = false;
	while(!result){
		sendAddressedVariable(address, &command, 1);
		result = tryReceiveWithTimeout(buffer, timeOut) and (uint32_t)(micros()-T_start) < timeOut;
	}
	return result;	
}

void RFM69HubNetwork::sendAwk(uint8_t address){
	constexpr uint8_t awk[RFM69_PACKAGE_LEN] = {RFM69_CTL_SENDACK};
	sendAddressed(address, (void*)&awk);
}

bool RFM69HubNetwork::waitForAwk(uint32_t timeOut){
	uint8_t buffer[RFM69_PACKAGE_LEN] = {0};
	uint32_t T_start = micros();
	bool result = receive_tryOnce(buffer);
	while(!result)
		receive_tryOnce(buffer);
		result = (buffer[0] == RFM69_CTL_SENDACK) and ((uint32_t)(micros()-T_start) < timeOut);
	return result;	
}
