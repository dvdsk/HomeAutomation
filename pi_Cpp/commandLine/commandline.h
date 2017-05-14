#ifndef COMMANDLINE
#define COMMANDLINE

#include <curses.h> //http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/keys.html
#include <menu.h>
#include <vector>
#include <ctime>
#include <string>
#include <sstream>
#include <cstdlib>//for calloc
#include <memory>
#include <atomic>

#include "../config.h"
#include "../dataStorage/PirData.h"
#include "../dataStorage/SlowData.h"
#include "../state/mainState.h"

//need to link with : -lmenu -lncurses

class CommandLineInterface{

	public:
	CommandLineInterface(std::shared_ptr<PirData> pirData_,
	                     std::shared_ptr<SlowData> slowData_,
											 SensorState* sensorState_);
	void mainMenu();

	private:
	std::shared_ptr<PirData> pirData;
	std::shared_ptr<SlowData> slowData;
	SensorState* sensorState;

	void sensor_values();

	void print_mainMenu(int highlight, const char* choices[], int n_choices);
	int mean(std::atomic<int>* array, int len);
};






#endif // MAINSTATE
