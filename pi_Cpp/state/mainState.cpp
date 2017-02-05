#include "mainState.h"

//decode url to a command to change a state or pass to a function
void MainState::httpSwitcher(const char* raw_url){

	if(0 == strcmp(raw_url, "/lamps")){
		std::cout<<"lamps call has been made\n";
		parseCommand(LIGHTS_ALLOFF);
	}
	return;
}

void MainState::parseCommand(Command toParse){
	
	switch(toParse){
		case LIGHTS_ALLON:
		
		break;
		
		case LIGHTS_ALLOFF:
		
		break;
		
		
	}
}

MainState::MainState(){
	
	userState_updated= std::make_shared<int>();
	
	lightValues = std::make_shared<std::array<int, 5>>();
	lightValues_mutex = std::make_shared<std::mutex>();
	
	userState = std::make_shared<user>();
	userState_mutex = std::make_shared<std::mutex>();
	
	lampOn = std::make_shared<std::array<bool, 6>>();
	
	*userState_updated = 0;
}

MainState::thread_watchForUpdate(){
	uint32_t CurrentTime;
	uint32_t lastBedMovement;
	
	while(true){ //can later be replaced with mutex to check if we should stop
		updateTime(&CurrentTime);
		
		switch(*majorState){
			case AWAY:
			update_away();			
			case SLEEPING:
			update_sleeping();
			break;
			case DEFAULT:
			update_default();
			break;
			case ALMOSTSLEEPING:
			update_almostSleeping();
			break;
		}
	}
}


