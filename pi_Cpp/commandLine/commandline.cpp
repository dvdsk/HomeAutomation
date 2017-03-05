#include "commandline.h"



#define WIDTH 30
#define HEIGHT 10 

//macro to find out the size of an array 
#define ARRAY_SIZE(a) (sizeof(a) / sizeof(a[0]))

int startx = 0;
int starty = 0;

CommandLineInterface::CommandLineInterface(std::shared_ptr<PirData> pirData_,
	std::shared_ptr<SlowData> slowData_){

	pirData = pirData_;
	slowData = slowData_;
}



void CommandLineInterface::mainMenu(){
	WINDOW *menu_win;
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
			//print sensor values
			break;
			case 3:
			clear();			
			graph_menu();
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
		case 3:				
			toAdd = HUMIDITY_BED;
			break;
		case 4:				
			toAdd = HUMIDITY_BATHROOM;
			break;
		case 5:				
			toAdd = HUMIDITY_DOORHIGH;
			break;
		case 6:				
			toAdd = BRIGHTNESS_BED;
			break;
		case 7:				
			toAdd = BRIGHTNESS_KITCHEN; //brightness window
			break;
		case 8:				
			toAdd = BRIGHTNESS_DOORHIGH; //brightness window
			break;
		case 9:				
			toAdd = BRIGHTNESS_BEYONDCURTAINS; //brightness window
			break;
		case 10:				
			toAdd = MOVEMENTSENSOR0; //brightness window
			break;
		//all other movementsensors....
	}
	return toAdd;
}

bool CommandLineInterface::fillPlotVector(MENU* my_menu, int n_choices, std::vector<plotables> toPlot){
	ITEM **items;

	items = menu_items(my_menu);	
	for(int i=0; i<n_choices; i++){
		if(item_value(items[i])){		
			toPlot.push_back(decodeMenu(i));
		}			
	}
}

uint32_t CommandLineInterface::unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

//TODO check sidewards scrolling possibility
void CommandLineInterface::graph_menu(){

	std::vector<plotables> toPlot;
	const char *choices[] = {
		"Bed", 	"Bathroom", 	"Door", " ", 	
		"Bed", 	"Bathroom", 	"Door", " ",
		"Bed", 	"Kitchen", 		"Door", 	"Window",
		"Bed left", 	"Bed right", 	"Heater", 	" ", 
		"Kitch. 1", 	"Kitch. 2",	"Kitch. 3", "Kitch. 4",
	 	"Bathroom left", 	"Bathroom right", " ", " ",
		"Co2", "Air Pressure", (char *)NULL,
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
	
	while((c = wgetch(stdscr)) != 10) { //10=KEY_ENTER
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
	mvprintw(12, 2, "Enter the range from now to plot followed by ENTER:");
	mvprintw(13, 2, "(format: days:hours:minutes:seconds)    ");	
	move(13, 42);	
	echo();	
	refresh();	

	char rawInput[80];	
	getstr(rawInput);
	std::istringstream input(rawInput);

	std::string days, hours, minutes, seconds;
	std::getline(input, days, ':');
	std::getline(input, hours, ':');
	std::getline(input, minutes, ':');
	std::getline(input, seconds);
	
	uint32_t now = unix_timestamp();
	int secondsAgo = std::stoi(days)*24*60*60 +std::stoi(hours)*60*60+
	                 std::stoi(minutes)*60 +std::stoi(seconds);

	Graph graph(toPlot, now-secondsAgo, now, pirData, slowData);

	/* Unpost and free all the memory taken up */
  unpost_menu(my_menu);
  free_menu(my_menu);
  for(i = 0; i < n_choices; ++i)
		free_item(my_items[i]);
	endwin();
}


