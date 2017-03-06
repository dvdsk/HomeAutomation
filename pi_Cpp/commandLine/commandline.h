#ifndef commandline
#define commandline

#include <curses.h> //http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/keys.html
#include <menu.h>
#include <vector>
#include <ctime>
#include <string>
#include <sstream>
#include <cstdlib>//for calloc
#include <memory>

#include "../config.h"
#include "../dataStorage/PirData.h"
#include "../dataStorage/SlowData.h"
#include "../state/mainState.h"
#include "../graph/MainGraph.h" 

//need to link with : -lmenu -lncurses

class CommandLineInterface{

	public:
	CommandLineInterface(std::shared_ptr<PirData> pirData_,
	                     std::shared_ptr<SlowData> slowData_,
											 std::shared_ptr<MainState> mainState_);
	void mainMenu();

	private:
	std::shared_ptr<PirData> pirData;
	std::shared_ptr<SlowData> slowData;
	std::shared_ptr<MainState> mainState;

	void graph_menu();
	void sensor_values();

	void print_mainMenu(int highlight, const char* choices[], int n_choices);
	int mean(int* array, const int len);

	void print_description();
	plotables decodeMenu(int menuChoice);
	bool fillPlotVector(MENU* my_menu, int n_choices, std::vector<plotables>& toPlot);
	uint32_t unix_timestamp();
};






#endif // MAINSTATE
