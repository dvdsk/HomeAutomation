#include RFM69HubNetwork.h



//bypass compiler and inline funct (comp doesnt know context)
__forceinline bool RFM69HubNetwork::receive_tryOnce_withAwk(uint8_t* buffer, uint8_t awkAddr){
	bool result;
	rfm.poll();
	if(result = rfm.available()){
		rfm.read(buffer);
		sendAwk(awkAddr);
	}
	return result;
}

__forceinline bool RFM69HubNetwork::receive_tryOnce(uint8_t* buffer){
	bool result;
	rfm.poll();

	if(result = rfm.available()){
		rfm.read(buffer);
	}
	return result;
}

void RFM69HubNetwork::RFM69HubNetwork(const char* encryptionKey, uint8_t hubAddr, uint32_t freq){
	setRecommended();
	setAES(false);
	setAesKey((void*)encryptionKey, (int)sizeof(encryptionKey));
	setPacketType(false, true);
	//AES is enabled, length below 16 results in zero padding
	//lengths shorter than 16 bytes not faster.
	setBufferSize(10);	
	setPacketLength(16); //bytes
	
	setNodeAddress(hubAddr);
	setFrequency(freq);
}

/* void RFM69HubNetwork::receive_tryForever_withAwk(uint8_t* buffer, uint8_t awkAddr){
	uint32_t T_start = timeMicroSec();
	while(!receive_tryOnce_withAwk(buffer, awkAddr) );
} */

bool RFM69HubNetwork::tryReceiveWithTimeout_sendAwk(uint8_t* buffer, uint32_t timeOut, uint8_t awkAddr){
	uint32_t T_start = timeMicroSec();
	bool result = receive_tryOnce_withAwk(buffer, awkAddr);
	while(!result)
		result = receive_tryOnce_withAwk(buffer, awkAddr) and ((uint32_t)(timeMicroSec()-T_start) < timeOut)
	return result;	
}

bool RFM69HubNetwork::tryReceiveWithTimeout(uint8_t* buffer, uint32_t timeOut){
	uint32_t T_start = timeMicroSec();
	bool result = receive_tryOnce(buffer);
	while(!result)
		result = receive_tryOnce(buffer) and ((uint32_t)(timeMicroSec()-T_start) < timeOut)
	return result;	
}

//bool RFM69HubNetwork::

bool RFM69HubNetwork::reSendCommandUntilAwknowledged_withTimeout(uint8_t command, uint8_t address, uint32_t timeOut){
	uint32_t T_start = timeMicroSec();
	do{
		sendAddressedVariable(address, &command, 1);
		waitForAwk(10);
	} while((uint32_t)(timeMicroSec()-T_start) < timeOut);
}

bool RFM69HubNetwork::reSendCommandUntilAnswered_withTimeout(uint8_t command, uint8_t address, uint8_t* buffer, uint32_t timeOut){
	uint32_t T_start = timeMicroSec();
	do{
		sendAddressedVariable(address, &command, 1);
		tryReceiveWithTimeout(buffer, timeOut);
	} while((uint32_t)(timeMicroSec()-T_start) < timeOut);
}

void RFM69HubNetwork::sendAwk(uint8_t address){
	constexpr uint8_t awk = RFM69_CTL_SENDACK;
	sendAddressedVariable(address, &awk, 1);
}

void RFM69HubNetwork::waitForAwk(uint32_t timeOut){
	uint8_t buffer[1];
	uint32_t T_start = timeMicroSec();
	bool result = receive_tryOnce(&buffer);
	while(!result)
		result = receive_tryOnce(&buffer) and ((uint32_t)(timeMicroSec()-T_start) < timeOut);
	return result;	
}



void RFM69HubNetwork::callFailed(){
	if(nRadioCalls<1000){
		radioCallFailed.set(nRadioCalls);
		nRadioCalls++;
	}
	else{
		if(pos>999) pos = 0;	
		radioCallFailed.set(pos);
		pos++;
	}
}

void RFM69HubNetwork::callSucceeded(){
	if(nRadioCalls<1000){
		//no reset needed as numb of succeeded calls = nRadioCalls
		nRadioCalls++;
	}
	else{
		if(pos>999) pos = 0;	
		radioCallFailed.reset(pos);
		pos++;
	}
}

uint16_t RFM69HubNetwork::getSucceeded(){
	return nRadioCalls - radioCallFailed.count();
}
uint16_t RFM69HubNetwork::getFailed(){
	return radioCallFailed.count();
}
float RFM69HubNetwork::getRatio(){
	return radioCallFailed.count()/(nRadioCalls - radioCallFailed.count());
}