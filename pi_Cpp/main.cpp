#include <iostream>
#include <fstream> //FIXME DEBUG
#include <ctime>
#include <signal.h>
#include <boost/exception/diagnostic_information.hpp> //for debugging

#include <time.h>
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

#include "smallFunct/sunSetRise.h"
#include "smallFunct/HttpsSocket.h"

#include "debug.h"

#include <stdio.h>//TODO DEBUG

const std::string PATHPIR = "pirs.binDat";
const int CACHESIZE_pir      = pirData::PACKAGESIZE*2;
const int CACHESIZE_slowData = slowData::PACKAGESIZE*2;

//cache for data
uint8_t cache1[CACHESIZE_pir];
uint8_t cache2[CACHESIZE_slowData];

FILE* file1; //needed as global for interrupt handling
FILE* file2;

std::mutex cv_updataSlow_m;
std::atomic<bool>* notShuttingdown = new std::atomic<bool>();
std::condition_variable cv_updataSlow;
////////////////////////////////////////////////////////////////////


void updateVSlow_thread(StateData* stateData){
  std::unique_lock<std::mutex> lk(cv_updataSlow_m);	
	constexpr double LLONGITUDE = 4.497010, LLATITUDE = 52.160114;
	double sunRise, sunSet;
	struct tm* aTime;
	time_t theTime;

	while(*notShuttingdown){

		theTime = time(NULL);
		aTime = localtime(&theTime);

		sun_rise_set(aTime->tm_year+1900, aTime->tm_mon+1, aTime->tm_mday, 
		LLONGITUDE, LLATITUDE, &sunRise, &sunSet);

		stateData->tWarm = 3600*(sunSet-1); 	//time since midnight in sec UTC
		stateData->tCool = 3600*(sunRise-1);	//time since midnight in sec UTC	

		cv_updataSlow.wait_for(lk, std::chrono::hours(24), 
		[notShuttingdown](){return !notShuttingdown->load();});
	}
}

void interruptHandler(int s){
  fflush(file1);
  fflush(file2);
  printf("Caught signal %d\n",s);
  exit(1); 
}

//FIXME REMOVE
uint32_t this_unix_timestamp() {
	time_t t = std::time(0);
	uint32_t now = static_cast<uint32_t> (t);
	return now;
}

int main(int argc, char* argv[])
{
	if(argc==2){	
		if(strcmp(argv[1], "startWakeup") == 0){		
			HttpsSocket local("127.0.0.1", config::HTTPSERVER_PORT);
			std::string request =
				 "GET /|state/wakeup HTTP/1.0\r\n"
				 "Host: 127.0.0.1\r\n"
				 "Accept: application/json\r\n"
				 "Connection: close\r\n"
				 "Authorization: Basic "
					+std::string(config::HTTPSERVER_USERPASS_B64)+"\r\n"
				 "\r\n\r\n";

			local.rawRequest(request);
			return 0;
		}
	}

	std::mutex* stopHttpServ = new std::mutex();
	TelegramBot* bot = new TelegramBot();
	SignalState* signalState = new SignalState;

	SensorState* sensorState = new SensorState;
	MpdState* mpdState = new MpdState;
	Mpd* mpd = new Mpd(mpdState, signalState);
	HttpState* httpState = new HttpState;
	ComputerState* computerState = new ComputerState;

	StateData* stateData = new StateData(sensorState, mpdState, mpd, httpState, computerState);

	PirData* pirDat = new PirData("pirs", cache1, CACHESIZE_pir);
	SlowData* slowDat = new SlowData("slowData", cache2, CACHESIZE_slowData);


	std::cout<<"test\n";
	(*stopHttpServ).lock();
	(*notShuttingdown) = true;
	file1 = pirDat->getFileP();
  file2 = slowDat->getFileP();

	/*start the http server that serves the telegram bot and
	  custom http protocol. NOTE: each connection spawns its 
	  own thread.*/	
	std::thread t1(thread_Https_serv, stopHttpServ, bot, httpState, signalState, pirDat, slowDat);
	std::cout<<"Https-Server started\n";

	std::thread t2(updateVSlow_thread, stateData);
	std::cout<<"Slow updating started\n";

	/*start the thread that checks the output of the arduino 
	  it is responsible for setting the enviremental variables
	  the statewatcher responds too*/
	std::thread t3(thread_checkSensorData, pirDat, slowDat, sensorState, signalState, notShuttingdown);
	std::cout<<"Sensor readout started\n";

	/*sleep to give checkSensorData time to aquire some data
	  from the arduino.*/
	std::cout<<"Waiting 5 seconds for sensors to set room states\n";
	//TODO FIXME std::this_thread::sleep_for(std::chrono::seconds(5));

	/*start the thread that is notified of state changes 
	  and re-evalutes the system on such as change. */
	std::thread t4(thread_state_management, notShuttingdown,stateData, signalState);
 	std::cout<<"State management started\n"; 

  signal(SIGINT, interruptHandler);  
	
	std::cout<<"cmd interface starting\n";
	std::this_thread::sleep_for(std::chrono::seconds(1));

	getchar();

//	TODO update commandlineinterface for new State system.
	CommandLineInterface interface(pirDat, slowDat, sensorState);
	interface.mainMenu();

//	getchar();
//	float y[MAXPLOTRESOLUTION];
//	uint32_t x[MAXPLOTRESOLUTION];
//	//uint32_t now = this_unix_timestamp();
//	int len = slowDat->fetchSlowData(1500379402, 1500379402+10000000, x, y, BRIGHTNESS_BED);

////	for(int i=0; i<len; i++)
////		if(x[i] == 0) std::cout<<"error at: i: "<<i<<"\n";

//  std::ofstream myfile;
//  myfile.open ("SlowData.txt");
//	for(int i=0; i<len; i++)
//  	myfile<<x[i]<<" "<<y[i]<<"\n";
//  myfile.close();

	//shutdown code
	(*stopHttpServ).unlock();
	(*notShuttingdown) = false;
	signalState->runUpdate();//(disadvantage) needs to run check to shutdown
	cv_updataSlow.notify_all();

	t1.join();
	t2.join();
	t3.join();
	t4.join();

	delete pirDat;
	delete slowDat;

	delete signalState;
	delete sensorState;
	delete httpState;
	delete computerState;
	delete mpdState;
	delete mpd;
  
	return 0;
}
