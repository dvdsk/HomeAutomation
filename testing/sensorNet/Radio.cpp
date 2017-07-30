#include "Radio.h"

/*compile with: "g++ -std=c++14 Radio.cpp -L/usr/local/lib -lrf24"   */

int main(){
	NodeMaster nodeMaster;
	nodeMaster.updateNodes();
}

uint8_t addresses[][6] = {"1Node","2Node"}; //FIXME

NodeMaster::NodeMaster() : RF24(pin::RADIO_CE, pin::RADIO_CS){

	//initialise and configure radio
  begin();
  setAutoAck(true);            // Ensure autoACK is enabled
  //setPayloadSize(5);                

  //setRetries(15,15);            // Smallest time between retries, max no. of retries
	setPALevel(RF24_PA_MIN);	  
  setDataRate(RF24_250KBPS);
	setChannel(108);	           // 2.508 Ghz - Above most Wifi Channels

	openWritingPipe(NODE_BED::addr);	
	openReadingPipe(PIPE, NODE_CENTRAL::addr);	

  //printDetails();              // Dump the configuration of the rf unit for debugging
	stopListening(); //need to call even though never started
/*
// TESTING CODE 
	unsigned long time;
	unsigned long started_waiting_at;
	unsigned long got_time;
	bool ok;
	bool timeout;
	while(1){ //loopt
		stopListening();

		std::cout<<"Now sending\n";
		time = millis();
		
		ok = write( &time, sizeof(unsigned long) );
		if (!ok){
			printf("SENDING FAILED.\n");
		}
		startListening();
		started_waiting_at = millis();
		timeout = false;
		while ( !available() && !timeout ) {
			if (millis() - started_waiting_at > 200 )
				timeout = true;
		}

		if ( timeout ) printf("Failed, response timed out.\n");
		else{
			got_time;
			read( &got_time, sizeof(unsigned long) );
			printf("Got response %lu, round-trip delay: %lu\n",got_time,millis()-got_time);
		}
	}
*/

//	bool test;
//	unsigned long time;
//	time = millis();
//	stopListening();
//	openWritingPipe(NODE_BED::addr);	
//	test = write(&time, sizeof(unsigned long));

//	if(test == true) std::cout<<"write succesfull\n";
//	else std::cout<<"write unsuccesful\n";
//	while(1);
}


void NodeMaster::updateNodes(){
	bool succes;
	bool notshuttingDown = true;
	uint32_t now, last = unix_timestamp(); //seconds
  uint32_t start_t; //milliseconds

	//request all nodes to reinitialise, setting all theire variables to the
	//default values.
	start_t = timeMicroSec();
 	succes = true;
	do{
		succes = succes && request_Init(NODE_BED::addr);
		if(timeMicroSec()-start_t > MAXDURATION) {
			std::cerr<<"TIMEOUT COULD NOT INIT REMOTE NODES\n";
			while(1); break;
		}
	} while(!succes && notshuttingDown);	
	std::cout<<"NODES (RE-) INIT SUCCESFULLY\n";


	//loop unit shutdown
	while(notshuttingDown){

		//instruct nodes to start there high freq measurements, and wait for them
		//to respond with the outcome. If that outcome contains a status message that
		//the low freq data is also ready, request that data and wait for it.
		succes = requestAndListen_fast(NODE_BED::fBuf, NODE_BED::addr, NODE_BED::LEN_fBuf);
		now = unix_timestamp();
		if(succes){
			process_Fast(); 	
			if(slowRdy(NODE_BED::fBuf)){
				succes = requestAndListen_slowValue(NODE_BED::sBuf, NODE_BED::addr, NODE_BED::LEN_sBuf);
				if(succes){
					process_Slow();
				}
			}
		}
		else {std::cout<<"rqFast failed!\n";}

		//instruct nodes to start there low freq measurements
		if(now-last >= 5){//every 5 seconds do this loop
			last = now;
			start_t = timeMicroSec();
			succes = false;
			do{
				succes = request_slowMeasure(NODE_BED::addr);
				if(timeMicroSec()-start_t > MAXDURATION) {
					std::cerr<<"TIMEOUT COULD NOT REQUEST SLOW-MEASURE\n";
					while(1); break;
				}
			} while(!succes && notshuttingDown);
		}
	}
}



bool NodeMaster::waitForReply(){
  uint32_t start_t;

	startListening(); 
  start_t = timeMicroSec();
  bool timeout = false;
	while ( !available() ){
		if (timeMicroSec() - start_t > MAXDURATION ){
      timeout = true;
			break;
		}
		//TODO introduce some sort of wait to prevent this from eating all of the
		//cpu. Should be slightly more then 1/2 the time it takes to respond normally
	}
	stopListening();
	return timeout;
}

bool NodeMaster::request_Init(const uint8_t addr[]){
	openWritingPipe(addr);
	return write(&headers::RQ_INIT, 1);
}


bool NodeMaster::request_slowMeasure(const uint8_t addr[]){
	openWritingPipe(addr);
	return write(&headers::RQ_MEASURE_SLOW, 1);
}

/* TODO use awk package? */
bool NodeMaster::slowRdy(const uint8_t buffer[]){
	uint8_t status = buffer[0];
	if(status == status::SLOW_RDY) return true;
	return false;
}


//TODO may not take more then 100 millisec
bool NodeMaster::requestAndListen_fast(uint8_t buffer[], 
     const uint8_t addr[], uint8_t replyLen)
{
	bool gotReply;
	openWritingPipe(addr);
	if(write(&headers::RQ_FAST, 1)){
		gotReply = waitForReply();
		if(gotReply){
			read(buffer, replyLen);
			return true;
		}
	}
	return false;
}

bool NodeMaster::requestAndListen_slowValue(uint8_t buffer[], 
     const uint8_t addr[], uint8_t replyLen)
{
	bool gotReply;
	openWritingPipe(addr);
	if(write(&headers::RQ_READ_SLOW, 1)){
		gotReply = waitForReply();
		if(gotReply){
			read(buffer ,replyLen);
			return true;
		}
	}
	return false;
}

uint32_t NodeMaster::unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

uint32_t NodeMaster::timeMicroSec(){
	timeval tv;	
	gettimeofday(&tv, nullptr);
	return tv.tv_usec;
}



/*Node side
//Node: 
	if no message recieved
		if data procedure running: continue
		else: go to deep sleep
	else
  	if request == fast
			check fast sensors
			transmit fast data + if(slow data aquired?)
		if request == slow
      start the procedure to aquire that data parallel to normal operation
	

*/

