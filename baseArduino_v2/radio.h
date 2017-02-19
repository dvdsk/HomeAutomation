#ifndef REMOTENODES_H
#define REMOTENODES_H

#include "Arduino.h"
#include "RF24.h"
#include "config.h"

class RemoteNodes : public RF24
{
public:
	RemoteNodes(uint16_t* fastData_, uint16_t* slowData_);
	//ask all nodes on the network for a slowUpdate next wireless polling run. The nodes only recieve the request 
	//the next polling round
	void requestSlowUpdate();
	//do a polling run, sending the data we want to recieve next 
	//and fetching the data from the previous request these runs should
	//thus be done often.
	void pollNodes();
private:
	bool outstandingSlowUpdate_N1;
	bool outstandingSlowUpdate_N2;

	uint8_t currentRq_N1;
	uint8_t currentRq_N2;
	
	//check the request
	void poll_N1();
	void poll_N2();

	void handleSlowData_N1();
	void handleSlowData_N2();

	void handleFastData_N1();
	void handleFastData_N2();

	uint16_t* fastData;
	uint16_t* slowData;
};

#include <Arduino.h> //needed for Serial.print

#endif

