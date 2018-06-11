#ifndef RFM69HUBNETWORK_H
#define RFM69HUBNETWORK_H

#include <bitset>
#include "plainRFM69.h"
#include "bareRFM69_const.h"
#include <stdint.h>
#include <iostream>

constexpr uint8_t RFM69_CTL_SENDACK = 0x80;
constexpr uint8_t RFM69_PACKAGE_LEN = 16;

class RFM69HubNetwork : public plainRFM69 {
	public:
		RFM69HubNetwork(const char* _encryptionKey, uint8_t _hubAddr, uint32_t _freq);
		void init();
		bool tryReceiveWithTimeout_sendAwk(uint8_t* buffer, uint32_t timeOut, uint8_t awkAddr);
		bool tryReceiveWithTimeout(uint8_t* buffer, uint32_t timeOut);
		
		bool SendCommandUntilAwknowledged_withTimeout(uint8_t command, uint8_t address, uint32_t timeOut);
		bool SendCommandUntilAnswered_withTimeout(uint8_t command, uint8_t address, uint8_t* buffer, uint32_t timeOutInBetween, uint8_t nTries);
		
		uint16_t getSucceeded();
		uint16_t getFailed();
		float getRatio();		

	private:
		const char* encryptionKey;
		uint8_t hubAddr;
		uint32_t freq;
	
		std::bitset<1000> radioCallFailed; //init as all unset;
		uint16_t pos = 0;//check if needed
		uint16_t nRadioCalls = 0;
		
		void callFailed();
		void callSucceeded();
		
		void sendAwk(uint8_t address);
		bool waitForAwk(uint32_t timeOut);
		
		__attribute__((always_inline)) bool receive_tryOnce_withAwk(uint8_t* buffer, uint8_t awkAddr);
		__attribute__((always_inline)) bool receive_tryOnce(uint8_t* buffer);
	};

#endif
