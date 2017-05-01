#include "commandline.h"



#define WIDTH 30
#define HEIGHT 10 

//macro to find out the size of an array 
#define ARRAY_SIZE(a) (sizeof(a) / sizeof(a[0]))

int startx = 0;
int starty = 0;

CommandLineInterface::CommandLineInterface(std::shared_ptr<PirData> pirData_,
	std::shared_ptr<SlowData> slowData_, std::shared_ptr<MainState> mainState_){

	pirData = pirData_;
	slowData = slowData_;
	state =mainState_;
}



void CommandLineInterface::mainMenu(){
	int highlight = 1;
	int choice = 0;
	int c;
	bool exit = false;

	const char* choices[] = {"System Info",	"Sensor values",
		                       "Https Server", "Exit", };
	
	int n_choices = sizeof(choices) / sizeof(char *);

	initscr();
	clear();
	noecho();
	cbreak();	/* Line buffering disabled. pass on everything */
	keypad(stdscr, TRUE); //init keypad for standard screen

	mvprintw(0,2, "Menu");
	refresh();
	print_mainMenu(highlight, choices, n_choices);
	while(1) {	
		c = getch();
		switch(c)	{	
			case KEY_UP: //key up
				if(highlight == 1)
					highlight = n_choices;
				else
					--highlight;
				break;
			case KEY_DOWN: //key down
				if(highlight == n_choices)
					highlight = 1;
				else 
					++highlight;
				break;
			case 10:
				choice = highlight;
				break;
			case 'q':
				choice = 4;//choice 5=exit
				break;
			default:
				mvprintw(24, 0, "Charcter pressed is = %3d Hopefully it can be printed as '%c'", c, c);
				refresh();
				break;
		}
		switch(choice){
			case 1:
			//do syst info
			break;
			case 2:
			clear();
			sensor_values();
			choice = 0;
			break;
			case 3:
			clear();			
			choice = 0;
			break;
			case 4:
			clear();	
			exit=true;			
			break;
		}
		print_mainMenu(highlight, choices, n_choices);

		if(exit){	/* User did a choice come out of the infinite loop */			
			break;
		}
	}	
	clrtoeol();
	refresh();
	endwin();
}

void CommandLineInterface::print_mainMenu(int highlight, const char* choices[], int n_choices){
	int x, y, i;	

	x = 2;
	y = 2;
	for(i = 0; i < n_choices; ++i){	
		if(highlight == i + 1){ /* High light the present choice */
			attron(A_STANDOUT);			
			mvprintw(y, x, "%s", choices[i]);
			attroff(A_STANDOUT);
		}
		else
			mvprintw(y, x, "%s", choices[i]);
		++y;
	}
	refresh();
}

int getdigit(const char* unit){
	timeout(-1);//no timeout	
	char rawInput[4];
	rawInput[0] = 0;			
	
	mvprintw(13, 2, unit);		
	move(13, 10);	
	clrtobot();	//clear text that was after the cursor

	refresh();	
	getstr(rawInput);

	if(rawInput[0] != 0){
		return std::stoi((std::string)rawInput);
	}
	return 0;
}

int CommandLineInterface::mean(int* array, int len){
	int mean = 0;
	for(int i=0; i<len; i++) {mean += *(array+0);}
	
	return mean/len;
}

void CommandLineInterface::sensor_values(){
	char c;	
	
	constexpr int COL1 = 2;
	constexpr int COL2 = 17;
	constexpr int COL3 = 23;
	constexpr int COL4 = 30;
	constexpr int COL5 = 37;

	mvprintw(1, COL1,  "Sensor type:");

	mvprintw(1, COL2, "now |");
	mvprintw(1, COL3, "5min |");
	mvprintw(1, COL4, "15min");
	mvprintw(1, COL5, "unit");

	mvprintw(3, COL1, "Temperature:");
	mvprintw(4, COL1, "Humidity:");
	mvprintw(5, COL1, "Brightness:");
	mvprintw(6, COL1, "Co2:");
	mvprintw(7, COL1, "Air Pressure:");

	mvprintw(3, COL5, "(deg Celcius)");
	mvprintw(4, COL5, "(%)");
	mvprintw(5, COL5, "(-)");
	mvprintw(6, COL5, "(ppm)");
	mvprintw(7, COL5, "(Pa)");

	timeout(1000);//timeout of 1 second for blocking calls
	mvprintw(LINES - 2, 2, "Enter to Exit, F to export to txt");
	
	do{
	{
		std::lock_guard<std::mutex> guard(state->sensorVal_mutex);		
		mvprintw(3, COL2, "%.1f", ((float)mean(state->tempValues, 
			                          temp::LEN))/10-10 );
		mvprintw(4, COL2, "%.1f", ((float)mean(state->humidityValues,
			                          hum::LEN))/10 );
		mvprintw(5, COL2, "%d", mean(state->lightValues, lght::LEN));
		mvprintw(6, COL2, "%d", state->CO2ppm);
		mvprintw(7, COL2, "%.1f", (state->Pressure/5.0+MINIMUM_MEASURABLE_PRESSURE));
		mvprintw(8, COL2, "%.1f", (state->Pressure));
	}
	mvprintw(7, COL2, "%d", 5);

	mvprintw(3, COL3, "%d", 5);
	mvprintw(4, COL3, "%d", 5);
	mvprintw(5, COL3, "%d", 5);
	mvprintw(6, COL3, "%d", 5);
	mvprintw(7, COL3, "%d", 5);

	mvprintw(3, COL4, "%d", 5);
	mvprintw(4, COL4, "%d", 5);
	mvprintw(5, COL4, "%d", 5);
	mvprintw(6, COL4, "%d", 5);
	mvprintw(7, COL4, "%d", 5);
	
	c = wgetch(stdscr);
	mvprintw(24, 0, "Charcter pressed is = %3d Hopefully it can be printed as '%c'", c, c);
	refresh();

	if(c == 'F'){	
		slowData->exportAllSlowData(0, -1);
		mvprintw(LINES-3, 0, "--> file exported succesfully to: 'SlowData.txt'");
		refresh();
	}
	
	}while(c != 10 && c != 113);

	clear();
	//MENU TO SELECT DETAILED SENSOR VALUES
}
