#ifndef RFM69HUBNETWORK_H
#define RFM69HUBNETWORK_H

#include <bitset>
#include "plainRFM69.h"

constexpr uint8_t RFM69_CTL_SENDACK = 0x80

class RFM69HubNetwork : public plainRFM69{
	public:
		RFM69HubNetwork(const char* encryptionKey, uint8_t hubAddr, uint32_t freq);
		bool tryReceiveWithTimeout_sendAwk(uint8_t* buffer, uint32_t timeOut, uint8_t awkAddr);
		bool tryReceiveWithTimeout(uint8_t* buffer, uint32_t timeOut, uint8_t awkAddr);
		
		reSendCommandUntilAwknowledged_withTimeout(uint8_t command, uint8_t address, uint32_t timeOut);
		reSendCommandUntilAnswered_withTimeout(uint8_t command, uint8_t address, uint32_t timeOut);
		
		uint16_t getSucceeded();
		uint16_t getFailed();
		float getRatio();		

	private:
		std::bitset<1000> radioCallFailed; //init as all unset;
		pos = 0;//check if needed
		nRadioCalls = 0;
		
		void callFailed();
		void callSucceeded();
		
		inline bool receiveWithAwk_tryOnce_withAwk(uint8_t* buffer, uint8_t awkAddr);
		inline bool receiveWithAwk_tryOnce_noAwk(uint8_t* buffer, uint8_t awkAddr);
		}

#endif
