#include "Radio.h"

/*compile with: "g++ -std=c++14 Radio.cpp -L/usr/local/lib -lrf24"   */

int main(){
	NodeMaster nodeMaster;
	//nodeMaster.updateNodes();
}

uint8_t addresses[][6] = {"1Node","2Node"}; //FIXME

NodeMaster::NodeMaster() : RF24(pin::RADIO_CE, pin::RADIO_CS){

	//initialise and configure radio
  begin();
  //setAddressWidth(3);          //sets adress with to 3 bytes long
  //setAutoAck(true);            // Ensure autoACK is enabled
  //setPayloadSize(5);                

  //setRetries(15,15);            // Smallest time between retries, max no. of retries
	setPALevel(RF24_PA_MIN);	  
  //setDataRate(RF24_250KBPS);
	setChannel(108);	           // 2.508 Ghz - Above most Wifi Channels

	openWritingPipe(addresses[1]);//NODE_BED::addr);	
	openReadingPipe(PIPE, addresses[0]);//NODE_CENTRAL::addr);	

  //openWritingPipe(addresses[1]);
  //openReadingPipe(1,addresses[0]);

	startListening();            // Start listening  
  //printDetails();              // Dump the configuration of the rf unit for debugging

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
}

/*Server side
	forever:

	if(send request fast == done)
		listen for awnser(timeout)
		process awns (if awns given)

	if(time>5 seconds)
		if(send request slow == done)
			listen for awnser(timeout) 
	while(slowNotComplete)
		do another fast check
		if(send request slow == done)
			listen for awnser(timeout)

funct notation:
*/

void NodeMaster::updateNodes(){
	bool succes = true;
	bool notshuttingDown = true;
	uint32_t now, last = unix_timestamp(); //seconds
  uint32_t start_t; //milliseconds

	//request all nodes to reinitialise, set all theire variables to theire
	//default values.
	bool test = false; //FIXME
	start_t = timeMicroSec();
	do{
		succes = request_Init(NODE_BED::addr); 
		//succes = succes && request_Init(NODE_BED::addr);
		std::cout<<"succes: "<<succes<<"\n";
		if(timeMicroSec()-start_t > MAXDURATION && false) {
			std::cerr<<"TIMEOUT COULD NOT INIT REMOTE NODES\n";
			while(1);
			break;
		}
	} while(!succes && notshuttingDown);	

	while(notshuttingDown){
		succes = requestAndListen_fast(NODE_BED::fBuf, NODE_BED::addr, NODE_BED::LEN_fBuf);
		now = unix_timestamp();
		if(succes){
			succes = false;
			process_Fast(); 	
			if(slowRdy(NODE_BED::fBuf)){
				succes = requestAndListen_slowValue(NODE_BED::sBuf, NODE_BED::addr, NODE_BED::LEN_sBuf);
				if(succes){
					succes = false; 
					process_Slow();
				}
			}
		}
		if(now-last >= 5){//every 5 seconds do this loop
			last = now;
			start_t = timeMicroSec();
			do{
				succes = request_slowMeasure(NODE_BED::addr);
				//TODO sleep for some time, //TODO optimise for multiple nodes
			} while(!succes && (timeMicroSec()-start_t < MAXDURATION));
			succes = false;
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
	bool test;
	std::cout<<"ran request init\n";
	openWritingPipe(addr);

	write(&headers::RQ_INIT, 1);


//	test = write(&headers::RQ_INIT, 1);
	if(test == true) std::cout<<"write succesfull\n";
	else std::cout<<"write unsuccesful\n";

	//std::cout<<"test: "<<test<<"\n";
	return test;
}


bool NodeMaster::request_slowMeasure(const uint8_t addr[]){
	openWritingPipe(addr);
	return (write(&headers::RQ_MEASURE_SLOW, 1));
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

