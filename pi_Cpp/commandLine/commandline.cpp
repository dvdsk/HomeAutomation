#include "commandline.h"

#define WIDTH 30
#define HEIGHT 10 

int startx = 0;
int starty = 0;

char *choices[] = { 
			"Choice 1",
			"Choice 2",
			"Choice 3",
			"Choice 4",
			"Exit",
		  };
int n_choices = sizeof(choices) / sizeof(char *);
void print_menu(int highlight);

int main()
{	WINDOW *menu_win;
	int highlight = 1;
	int choice = 0;
	int c;

	initscr();
	clear();
	noecho();
	cbreak();	/* Line buffering disabled. pass on everything */
	keypad(stdscr, TRUE); //init keypad for standard screen

	mvprintw(0, 0, "Use arrow keys to go up and down, Press enter to select a choice");
	refresh();
	print_menu(highlight);
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
		print_menu(highlight);
		if(choice != 0)	/* User did a choice come out of the infinite loop */
			break;
	}	
	mvprintw(23, 0, "You chose choice %d with choice string %s\n", choice, choices[choice - 1]);
	clrtoeol();
	refresh();
	endwin();
	return 0;
}


void print_menu(int highlight)
{
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
