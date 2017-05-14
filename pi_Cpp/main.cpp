#include <iostream>
#include <ctime>
#include <signal.h>
#include <boost/exception/diagnostic_information.hpp> //for debugging

#include <chrono>
#include <thread>
#include <mutex>
#include <memory>
#include <atomic>

#include "config.h"
#include "arduinoContact/Serial.h"
#include "arduinoContact/decode.h"
#include "dataStorage/MainData.h"
#include "dataStorage/PirData.h"
#include "state/mainState.h"
#include "state/stateManagement.cpp" //<<<

#include "telegramBot/telegramBot.h"
#include "httpServer/mainServer.h"
#include "commandLine/commandline.h"
#include "mpd/mpd.h"

#include "debug.h"

const std::string PATHPIR = "pirs.binDat";
const int CACHESIZE_pir      = pirData::PACKAGESIZE*2;
const int CACHESIZE_slowData = slowData::PACKAGESIZE*2;

//cache for data
uint8_t cache1[CACHESIZE_pir];
uint8_t cache2[CACHESIZE_slowData];

FILE* file1; //needed as global for interrupt handling
FILE* file2;

void interruptHandler(int s){
  fflush(file1);
  fflush(file2);
  printf("Caught signal %d\n",s);
  exit(1); 
}

int main(int argc, char* argv[])
{
	std::shared_ptr<std::mutex> stopHttpServ = std::make_shared<std::mutex>();
	std::shared_ptr<std::atomic<bool>> notShuttingdown = std::make_shared<std::atomic<bool>>();

	std::shared_ptr<TelegramBot> bot = std::make_shared<TelegramBot>();

	//expriment not using shared pointers (possible speedup)
	SignalState* signalState = new SignalState;
	SensorState* sensorState = new SensorState;
	HttpState* httpState = new HttpState;
	MpdState* mpdState = new MpdState;

	std::shared_ptr<Mpd> mpd = std::make_shared<Mpd>(mpdState, signalState);

	std::shared_ptr<PirData> pirData = std::make_shared<PirData>("pirs", cache1, CACHESIZE_pir);
	std::shared_ptr<SlowData> slowData = std::make_shared<SlowData>("slowData", cache2, CACHESIZE_slowData);

	(*stopHttpServ).lock();
	(*notShuttingdown) = true;
	file1 = pirData->getFileP();
  file2 = slowData->getFileP();

	#ifndef TEST
	/*start the http server that serves the telegram bot and
	  custom http protocol. NOTE: each connection spawns its 
	  own thread.*/	
	std::thread t1(thread_Https_serv, stopHttpServ, bot, httpState, signalState, pirData, slowData);
	std::cout<<"Https-Server started\n";

	/*start thread to recieve updates if mpd status changes*/
	std::thread t2(thread_Mpd_readLoop, mpd, notShuttingdown);
	std::cout<<"Mpd-Server started\n";

	/*start the thread that checks the output of the arduino 
	  it is responsible for setting the enviremental variables
	  the statewatcher responds too*/
	std::thread t3(thread_checkSensorData, pirData, slowData, sensorState, signalState, notShuttingdown);
	std::cout<<"Sensor readout started\n";

	/*sleep to give checkSensorData time to aquire some data
	  from the arduino.*/
	std::cout<<"Waiting 5 seconds for sensors to set room states\n";
	std::this_thread::sleep_for(std::chrono::seconds(5));

	/*start the thread that is notified of state changes 
	  and re-evalutes the system on such as change. */
	Mpd* mpd2 = mpd.get(); //TODO remove smart pointers	
	std::thread t4(thread_state_management, notShuttingdown, signalState, 
	  sensorState, mpdState, mpd2, httpState);
 	std::cout<<"State management started\n"; 

  signal(SIGINT, interruptHandler);  
	
	std::cout<<"cmd interface starting\n";
	std::this_thread::sleep_for(std::chrono::seconds(1));

	getchar();
	signalState->runUpdate();

	getchar();
	signalState->runUpdate();

	getchar();
	signalState->runUpdate();

	getchar();

//	TODO update commandlineinterface for new State system.
//	CommandLineInterface interface(pirData, slowData, state);
//	interface.mainMenu();

//	slowData->exportAllSlowData(1492048892, -1);

	//shutdown code
	(*stopHttpServ).unlock();
	(*notShuttingdown) = false;
	signalState->runUpdate();//(disadvantage) needs to run check to shutdown

	t1.join();
	t2.join();
	t3.join();
	t4.join();
	#endif

	#ifdef TEST
	debug(pirData,	slowData, state);
	#endif
  
	return 0;
}
