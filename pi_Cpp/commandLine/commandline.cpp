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
	mainState =mainState_;
}



void CommandLineInterface::mainMenu(){
	int highlight = 1;
	int choice = 0;
	int c;
	bool exit;

	const char* choices[] = {"System Info",	"Sensor values",	"Graph Sensor Data",
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
				choice = 5;//choice 5=exit
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
			graph_menu();
			choice = 0;
			break;
			case 5:
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

void CommandLineInterface::print_description(){
	//do something with "current-item"

}

plotables CommandLineInterface::decodeMenu(int menuChoice){
	plotables toAdd;
	switch(menuChoice){
		case 0:				
			toAdd = TEMP_BED;
			break;
		case 1:				
			toAdd = TEMP_BATHROOM;
			break;
		case 2:				
			toAdd = TEMP_DOORHIGH;
			break;
		case 4:				
			toAdd = HUMIDITY_BED;
			break;
		case 5:				
			toAdd = HUMIDITY_BATHROOM;
			break;
		case 6:				
			toAdd = HUMIDITY_DOORHIGH;
			break;
		case 8:				
			toAdd = BRIGHTNESS_BED;
			break;
		case 9:				
			toAdd = BRIGHTNESS_KITCHEN; //brightness window
			break;
		case 10:				
			toAdd = BRIGHTNESS_DOORHIGH; //brightness window
			break;
		case 11:				
			toAdd = BRIGHTNESS_BEYONDCURTAINS; //brightness window
			break;
		case 12:				
			toAdd = MOVEMENTSENSOR0; //brightness window
			break;
		//all other movementsensors....
		case 24:				
			toAdd = CO2PPM; //brightness window
			break;
		case 23:				
			toAdd = CO2PPM; //brightness window
			break;
		case 22:				
			toAdd = CO2PPM; //brightness window
			break;
	}
	return toAdd;
}

void CommandLineInterface::fillPlotVector(MENU* my_menu, int n_choices, std::vector<plotables>& toPlot){
	ITEM **items;
	items = menu_items(my_menu);	
	for(int i=0; i<n_choices; i++){
		if(item_value(items[i]) == 1){		
			toPlot.push_back(decodeMenu(i));
		}			
	}
}

uint32_t CommandLineInterface::unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
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

//TODO check sidewards scrolling possibility
void CommandLineInterface::graph_menu(){

	std::vector<plotables> toPlot;
	const char *choices[] = {
		"Bed", 	"Bathroom", 	"Door", " ", 								//0-3
		"Bed", 	"Bathroom", 	"Door", " ",								//4-7
		"Bed", 	"Kitchen", 		"Door", 	"Window",					//8-11
		"Bed left", 	"Bed right", 	"Heater", 	" ", 			//12-15
		"Kitch. 1", 	"Kitch. 2",	"Kitch. 3", "Kitch. 4",	//16-19
	 	"Bathroom left", 	"Bathroom right", " ", " ",			//20-23
		"Co2", "Air Pressure", (char *)NULL,							//24-25
	};

	ITEM **my_items;
	int c;				
	MENU *my_menu;
  int n_choices, i;
	
	/* Initialize curses */
	initscr();
	//start_color(); //also turns everything black....
	cbreak();
  noecho();
	keypad(stdscr, TRUE);
	init_pair(1, COLOR_RED, COLOR_BLACK);
	init_pair(2, COLOR_CYAN, COLOR_BLACK);

	/* Create items */
  n_choices = ARRAY_SIZE(choices);
  my_items = (ITEM **)calloc(n_choices, sizeof(ITEM *));
  for(i = 0; i < n_choices; ++i)
    my_items[i] = new_item(choices[i], choices[i]);

	/* Crate menu */
	my_menu = new_menu((ITEM **)my_items);

	/* Set menu option not to show the description */
	menu_opts_off(my_menu, O_SHOWDESC | O_ONEVALUE);
     
	/* Set main window and sub window */
  set_menu_win(my_menu, stdscr);
  set_menu_sub(my_menu, derwin(stdscr, 9, 54, 3, 18)); //height, with, start y, start x
	set_menu_format(my_menu, 9, 4);//menu rows, collums
	set_menu_mark(my_menu, "");

	/* Print a border around the main window and print a title */
  //box(my_menu_win, 0, 0);
	
	attron(COLOR_PAIR(2));
	mvprintw(LINES - 2, 2, "Use Arrow Keys to navigate, Space to select and Enter to Continue or Exit");
	attroff(COLOR_PAIR(2));
	
	//print text for window
	mvprintw(1, 2, "Select values to plot:");
	mvprintw(3, 2, "Temperature:");
	mvprintw(4, 2, "Humidity:");
	mvprintw(5, 2, "Brightness:");
	mvprintw(6, 2, "Movement:");
	mvprintw(9, 2, "Other sensors:");
	refresh();

	/* Post the menu */
	post_menu(my_menu);
	refresh();
	
	while((c = wgetch(stdscr)) != 10 && c !='q' ) { //10=KEY_ENTER
  	switch(c) {
			case KEY_DOWN:
				menu_driver(my_menu, REQ_DOWN_ITEM);
				break;
			case KEY_UP:
				menu_driver(my_menu, REQ_UP_ITEM);
				break;
			case KEY_LEFT:
				menu_driver(my_menu, REQ_LEFT_ITEM);
				break;
			case KEY_RIGHT:
				menu_driver(my_menu, REQ_RIGHT_ITEM);
				break;
			case KEY_NPAGE:
				menu_driver(my_menu, REQ_SCR_DPAGE);
				break;
			case KEY_PPAGE:
				menu_driver(my_menu, REQ_SCR_UPAGE);
				break;
			case 32: 	//KEY_SPACE
				menu_driver(my_menu, REQ_TOGGLE_ITEM);
				break;
		}
  	refresh();
	}
	//fill the toplot items vector
	fillPlotVector(my_menu, n_choices, toPlot);
	if(toPlot.size()>0){	
		mvprintw(12, 2, "Enter the range from now to plot followed by ENTER:");
		echo();	


		uint32_t now = unix_timestamp();
		int secondsAgo = getdigit("days:")*24*60*60 +
		               	 getdigit("hours:")*60*60+
			               getdigit("minutes:")*60 +
			               getdigit("seconds:");

	}
	/* Unpost and free all the memory taken up */
	clear();  
	unpost_menu(my_menu);
  free_menu(my_menu);
  for(i = 0; i < n_choices; ++i)
		free_item(my_items[i]);
	endwin();
}

int CommandLineInterface::mean(int* array, int len){
	int mean = 0;
	len = 1;
	for(int i=0; i<len; i++) {mean = *(array+0);}
	
	return mean;
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
	mvprintw(6, COL5, "(ppm):");
	mvprintw(7, COL5, "");

	timeout(1000);//timeout of 1 second for blocking calls
	mvprintw(LINES - 2, 2, "Enter to Exit");
	
	do{
	mvprintw(3, COL2, "%.1f", ((float)mean(mainState->tempValues, 
	                            mainState::LEN_tempValues))/10-10 );
	mvprintw(4, COL2, "%.1f", ((float)mean(mainState->humidityValues,
	                            mainState::LEN_humidityValues))/10 );
	mvprintw(5, COL2, "%d", mean(mainState->lightValues, mainState::LEN_lightValues));
	mvprintw(6, COL2, "%d", mainState->CO2ppm);
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
	}while(c == -1);

	clear();
	//MENU TO SELECT DETAILED SENSOR VALUES
}
