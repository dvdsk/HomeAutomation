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
#include "../dataStorage/PirData.h" //TODO needed?
#include "../dataStorage/SlowData.h" //TODO needed?
#include "../graph/MainGraph.h" 

//need to link with : -lmenu -lncurses

class CommandLineInterface{

	public:
	CommandLineInterface(std::shared_ptr<PirData> pirData_,
	                     std::shared_ptr<SlowData> slowData_);
	void mainMenu();

	private:
	std::shared_ptr<PirData> pirData;
	std::shared_ptr<SlowData> slowData;

	void graph_menu();

	void print_mainMenu(int highlight, const char* choices[], int n_choices);

	void print_description();
	plotables decodeMenu(int menuChoice);
	bool fillPlotVector(MENU* my_menu, int n_choices, std::vector<plotables> toPlot);
	uint32_t unix_timestamp();
};






#endif // MAINSTATE
