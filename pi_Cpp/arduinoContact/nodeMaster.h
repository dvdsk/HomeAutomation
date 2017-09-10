#ifndef NODEMASTER_H
#define NODEMASTER_H

#ifdef __arm__ //only compile on raspberry pi

#include <cstdlib>
#include <iostream>

#include <RF24/RF24.h>
#include <sys/time.h>
#include <ctime>
#include <bitset>

#include "decode.h"
#include "radio.h"
#include "../encodingScheme.h"

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
	NodeMaster(PirData* pirData, SlowData* slowData,
	           SensorState* sensorState, SignalState* signalState);
	~NodeMaster();
	bool requestNodeInit(bool notshuttingDown, const uint8_t addr[]);
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

	//for threading
	std::thread* m_thread;
	std::atomic<bool> notshuttingDown;	
};

inline void thread_NodeMaster_updateNodes(NodeMaster* nodeMaster);

#endif
#endif
