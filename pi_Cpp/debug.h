#include <iostream>
#include <ctime>
#include <signal.h>
#include <boost/exception/diagnostic_information.hpp> //for debugging

#include <chrono>
#include <thread>
#include <mutex>
#include <memory>
#include <atomic>
#include <vector>

#include "config.h"
#include "arduinoContact/Serial.h"
#include "arduinoContact/decode.h"
#include "dataStorage/MainData.h"
#include "dataStorage/PirData.h"
#include "state/mainState.h"
#include "telegramBot/telegramBot.h"
#include "httpServer/mainServer.h"
#include "commandLine/commandline.h"

uint32_t this_unix_timestamp() {
	time_t t = std::time(0);
	uint32_t now = static_cast<uint32_t> (t);
	return now;
}

void debug(std::shared_ptr<PirData> pirData,	std::shared_ptr<SlowData> slowData, 
std::shared_ptr<MainState> mainState){

	uint32_t now = this_unix_timestamp();
	uint32_t secondsAgo = 60*60*24*1;
	
	uint32_t stopT = now;
	uint32_t startT = now-secondsAgo;
	double x[1000];
	double y[1000];	
	plotables i = TEMP_BED;
	std::vector<plotables> toPlot;
	toPlot.push_back(TEMP_BED);


	char ch;
	ch = getchar();


	std::cout<<"conf: "<<Enc_slow::TEMP_BED<<"\n";
	std::cout<<"test test\n";
	//std::cout<<"len: "<<len<<"\n\n";

}

