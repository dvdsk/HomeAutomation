#include "nodeMaster.h"
#ifdef __arm__

namespace NODE_BED{
	uint8_t fBuf[LEN_fBuf];
	uint8_t sBuf[LEN_sBuf];
	ConnectionStats conStats;
}
namespace NODE_KITCHEN{
	uint8_t fBuf[LEN_fBuf];
	uint8_t sBuf[LEN_sBuf];
	ConnectionStats conStats;
}
namespace NODE_BATHROOM{
	uint8_t fBuf[LEN_fBuf];
	uint8_t sBuf[LEN_sBuf];
	ConnectionStats conStats;
}

bool NodeMaster::requestNodeInit(bool notshuttingDown, const uint8_t addr[]){
 	bool succes = true;
	uint32_t start_t;

	start_t = timeMicroSec();
	do{
		succes = request_Init(addr);
		if(succes){NODE_BED::conStats.callSucceeded(); break;	}
		else NODE_BED::conStats.callFailed(); 

		if((uint32_t)(timeMicroSec()-start_t) > MAXDURATION) {
			std::cerr<<"COULD NOT INIT NODE AT ADDR: '"<<addr<<"'"
							 <<", check if node is online\n";
			break;
		}

	} while(!succes && notshuttingDown);

	if(succes){
		start_t = timeMicroSec();
		succes = false;
		do{
			succes = waitForReply();
			if((uint32_t)(timeMicroSec()-start_t) > MAXDURATION) {
				std::cerr<<"NO REPLY FROM NODE AT ADDR: '"<<addr<<"'"
					       <<", something might be wrong with the program on it\n";
				break;
			}	
		} while(!succes && notshuttingDown);
	}
	return succes;
}

NodeMaster::NodeMaster(PirData* pirData, SlowData* slowData,
	                     SensorState* sensorState, SignalState* signalState) 
	//: RF24(pin::RADIO_CE, pin::RADIO_CS), 
	: RF24(pin::RADIO_CE, 0*10+0),
		Decode(pirData, slowData, sensorState, signalState)
{
	bool succes = true;
	bool notshuttingDown = true;

	//initialise and configure radio
  begin();
  //setAutoAck(true);            // Ensure autoACK is enabled
  //setPayloadSize(5);                

  setRetries(1,15);            // Smallest time between retries, max no. of retries
	//setPALevel(RF24_PA_LOW);	  
	setPALevel(RF24_PA_HIGH);	
  setDataRate(RF24_250KBPS);	//RF24_250KBPS
	setChannel(108);	           // 2.508 Ghz - Above most Wifi Channels
	
	openReadingPipe(PIPE, NODE_CENTRAL::addr);	

  printDetails();              // Dump the configuration of the rf unit for debugging
	stopListening(); //need to call even though never started


	//request all nodes to reinitialise, setting all theire variables to the
	//default values.
	//TODO renable:	succes &= requestNodeInit(notshuttingDown, NODE_BED::addr);
	//succes &= requestNodeInit(notshuttingDown, NODE_BATHROOM::addr);
	succes &= requestNodeInit(notshuttingDown, NODE_KITCHEN::addr);

	if(succes){
		std::cout<<"ALL NODES (RE-) INIT SUCCESFULLY\n";
		m_thread = new std::thread(thread_NodeMaster_updateNodes, this);
	}
	else std::cout<<"EXITING NODEMASTER\n";
}

NodeMaster::~NodeMaster(){
	notshuttingDown = false;
	m_thread->join();
	delete m_thread;
}

inline void thread_NodeMaster_updateNodes(NodeMaster* nodeMaster)
{
	nodeMaster->updateNodes();
}

void NodeMaster::checkFast(uint32_t now){
	bool succes;
	{
	using namespace NODE_BED;		
		succes = requestAndListen_fast(fBuf, addr, LEN_fBuf);
		if(succes){
			conStats.callSucceeded();
			process_Fast_BED(now, fBuf); 	
			if(slowRdy(fBuf)){
				succes = requestAndListen_slowValue(sBuf, addr, LEN_sBuf);
				if(succes){
					conStats.callSucceeded();
					process_Slow_BED(now, sBuf);
				}
				else conStats.callFailed();
			}
		}
		else conStats.callFailed();
	}
	{
	using namespace NODE_BATHROOM;		
		succes = requestAndListen_fast(fBuf, addr, LEN_fBuf);
		if(succes){
			conStats.callSucceeded();
			process_Fast_BATHROOM(now, fBuf); 	
			if(slowRdy(fBuf)){
				succes = requestAndListen_slowValue(sBuf, addr, LEN_sBuf);
				if(succes){
					conStats.callSucceeded();
					process_Slow_BATHROOM(now, sBuf);
				}
				else conStats.callFailed();
			}
		}
		else conStats.callFailed();
	}
	{
	using namespace NODE_KITCHEN;		
		succes = requestAndListen_fast(fBuf, addr, LEN_fBuf);
		if(succes){
			conStats.callSucceeded();
			process_Fast_KITCHEN(now, fBuf); 	
			if(slowRdy(fBuf)){
				succes = requestAndListen_slowValue(sBuf, addr, LEN_sBuf);
				if(succes){
					conStats.callSucceeded();
					process_Slow_KITCHEN(now, sBuf);
				}
				else conStats.callFailed();
			}
		}
		else conStats.callFailed();
	}
}

void NodeMaster::rqSlowBathroom(uint32_t now){
	bool succes;
	bool notshuttingDown = true;
	uint32_t startedRq = timeMicroSec();
	{
	using namespace NODE_BATHROOM;
		succes = false;
		do{				
			succes = request_slowMeasure(addr);
			if(succes){
				conStats.callSucceeded();
				break;
			}
			else{
				conStats.callFailed();
				checkFast(now);				
			}
			if( (uint32_t)(timeMicroSec()-startedRq) > MAXDURATION*10){
				std::cout<<"BATHROOM TIMEOUT\n";
				break;
			}

		} while(notshuttingDown);//TODO
	}
}

void NodeMaster::rqSlowBed(uint32_t now){
	bool succes;
	bool notshuttingDown = true;
	uint32_t startedRq = timeMicroSec();
	{
	using namespace NODE_BED;
		succes = false;
		do{
			succes = request_slowMeasure(addr);
			if(succes){
				conStats.callSucceeded();
				break;
			}
			else{
				conStats.callFailed();
				checkFast(now);			
			}
			if( (uint32_t)(timeMicroSec()-startedRq) > MAXDURATION*10){
				//std::cout<<"SLOWBED TIMEOUT\n";
				break;
			}

		} while(notshuttingDown);//TODO
	}
}

void NodeMaster::updateNodes(){
	uint32_t now, last; //seconds
	now = last = unix_timestamp();

	//loop unit shutdown
	now = last;
	while(notshuttingDown){
		//instruct nodes to start there high freq measurements, and wait for them
		//to respond with the outcome. If that outcome contains a status message that
		//the low freq data is also ready, request that data and wait for it.
		
		checkFast(now);

		//instruct nodes to start there low freq measurements
		now = unix_timestamp();	
		if(now-last >= 5){//every 5 seconds do this loop	
			last = now;

			//take max 3*MAXDURATION << 1 second so now need not be fetched again
			rqSlowBathroom(now); 
			rqSlowBed(now); 
			//rqKitchen(now); //TODO when relevent	

		}//if
	}//while(notshuttingdown
}



bool NodeMaster::waitForReply(){
  uint32_t start_t;

	startListening(); 
  start_t = timeMicroSec();
  bool gotreply = true;
	while ( !available() ){
		if((uint32_t)(timeMicroSec()-start_t) >= MAXDURATION) {
      gotreply = false;
			break;
		}
		//TODO introduce some sort of wait to prevent this from eating all of the
		//cpu. Should be slightly more then 1/2 the time it takes to respond normally
	}
	stopListening();
	return gotreply;
}

bool NodeMaster::request_Init(const uint8_t addr[]){
	openWritingPipe(addr);
	return write(&headers::RQ_INIT, 1);
}


bool NodeMaster::request_slowMeasure(const uint8_t addr[]){
	openWritingPipe(addr);
	return write(&headers::RQ_MEASURE_SLOW, 1);
}

bool NodeMaster::slowRdy(const uint8_t buffer[]){
	uint8_t status = buffer[0];
	if(status & status::SLOW_RDY) return true;
	return false;
}

bool NodeMaster::requestAndListen_fast(uint8_t fBuf[], 
     const uint8_t addr[], uint8_t replyLen)
{
	bool gotReply;
	openWritingPipe(addr);
	if(write(&headers::RQ_FAST, 1)){
		gotReply = waitForReply();
		if(gotReply){
			read(fBuf, replyLen);
			return true;
		}
	}
	return false;
}

bool NodeMaster::requestAndListen_slowValue(uint8_t sBuf[], 
     const uint8_t addr[], uint8_t replyLen)
{
	bool gotReply;
	openWritingPipe(addr);
	if(write(&headers::RQ_READ_SLOW, 1)){
		gotReply = waitForReply();
		if(gotReply){
			read(sBuf, replyLen);
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

//only works for half a second (500 millisec) then overflow happens
uint32_t NodeMaster::timeMicroSec(){
	timeval tv;	
	gettimeofday(&tv, nullptr);
	return tv.tv_usec;
}


ConnectionStats::ConnectionStats(){
	pos = 0;//check if needed
	nRadioCalls = 0;
}

void ConnectionStats::callFailed(){
	if(nRadioCalls<1000){
		radioCallFailed.set(nRadioCalls);
		nRadioCalls++;
	}
	else{
		if(pos>999) pos = 0;	
		radioCallFailed.set(pos);
		pos++;
	}
//	std::cout<<"Failure: "
//					 <<( 100*(float)radioCallFailed.count()/
//							     (float)nRadioCalls )
//					 <<" %\n";
}

void ConnectionStats::callSucceeded(){
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

uint16_t ConnectionStats::getSucceeded(){
	return nRadioCalls - radioCallFailed.count();
}
uint16_t ConnectionStats::getFailed(){
	return radioCallFailed.count();
}
float ConnectionStats::getRatio(){
	return radioCallFailed.count()/(nRadioCalls - radioCallFailed.count());
}
#endif
