#ifndef RADIO_H
#define RADIO_H

#include <cstdlib>
#include <iostream>
#include <RF24/RF24.h>
#include <sys/time.h>
#include <ctime>
#include <bitset>

#include "decode.h"
#include "encodingScheme.h"

//TODO extra for debugging
#include <sstream>
#include <string>
#include <unistd.h>

class ConnectionStats{
	public:
		ConnectionStats();
		void callFailed();
		void callSucceeded();
		uint16_t getSucceeded();
		uint16_t getFailed();
		uint16_t getRatio();
	private:	
		std::bitset<1000> radioCallFailed; //init as all unset;
		int nRadioCalls = 0; //goes up to 1000
		int pos = 0;
};

class NodeMaster : public RF24, Decode
{
public:
	NodeMaster();
	void updateNodes();
private:
	bool request_Init(const uint8_t addr[]);
	bool waitForReply();

	bool requestAndListen_fast(uint8_t buffer[], const uint8_t addr[], 
	     uint8_t replyLen);

	bool request_slowMeasure(const uint8_t addr[]);
	bool slowRdy(const uint8_t buffer[]);
	bool requestAndListen_slowValue(uint8_t buffer[], const uint8_t addr[],
	     uint8_t replyLen);

	uint32_t unix_timestamp();
	uint32_t timeMicroSec();
};

namespace NODE_BED{
	constexpr uint8_t addr[] = "2Node"; //addr may only diff in first byte
	constexpr uint8_t LEN_fBuf = EncFastArduino::LEN_BED_NODE;
	constexpr uint8_t LEN_sBuf = EncSlowArduino::LEN_BEDNODE;

	uint8_t fBuf[LEN_fBuf];
	uint8_t sBuf[LEN_sBuf];
	ConnectionStats conStats;
}

namespace NODE_KITCHEN{
	constexpr uint8_t addr[] = "3Node"; //addr may only diff in first byte
	constexpr uint8_t LEN_fBuf = 10;
	constexpr uint8_t LEN_sBuf = 10;

	ConnectionStats conStats;

	uint8_t fBuf[EncSlowArduino::LEN_BEDNODE];
	uint8_t sBuf[EncFastArduino::LEN_BED_NODE];
}

//time in which node must reply through awk package.
constexpr int MAXDURATION = 500*1000; //milliseconds

namespace pin{
	constexpr int RADIO_CE = 22;
	constexpr int RADIO_CS = 0;
}

namespace status{
	constexpr uint8_t SLOW_RDY = 1;
}

namespace headers{
	constexpr uint8_t RQ_FAST = 0;
	constexpr uint8_t RQ_MEASURE_SLOW = 1;
	constexpr uint8_t RQ_READ_SLOW = 2;
	constexpr uint8_t RQ_INIT = 3;
}

constexpr uint8_t PIPE = 1;

namespace NODE_CENTRAL{
	constexpr uint8_t addr[] = "1Node"; //addr may only diff in first byte
}



//pasts together all the data
void process_Slow(uint32_t now){
	std::cout<<"processed-slow\n";
}

//should be node specific
void process_Fast(){
	//std::cout<<"processed-fast\n";
}

#endif

