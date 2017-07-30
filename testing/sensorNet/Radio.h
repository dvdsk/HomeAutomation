#ifndef RADIO_H
#define RADIO_H

#include <cstdlib>
#include <iostream>
#include <RF24/RF24.h>
#include <sys/time.h>
#include <ctime>

//TODO extra for debugging
#include <sstream>
#include <string>
#include <unistd.h>

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
	constexpr uint8_t addr[] = "0No";
}

namespace NODE_BED{
	constexpr uint8_t addr[] = "1No";
	constexpr uint8_t LEN_fBuf = 10;
	constexpr uint8_t LEN_sBuf = 10;

	uint8_t fBuf[LEN_fBuf];
	uint8_t sBuf[LEN_sBuf];
}
//time in which node must reply through awk package.
constexpr int MAXDURATION = 50*1000; //milliseconds

class NodeMaster : public RF24 
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

void process_Slow(){
	std::cout<<"test\n";
}
void process_Fast(){
	std::cout<<"test\n";
}

#endif

