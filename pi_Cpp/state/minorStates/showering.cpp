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
		
		switch(*userState){
			case not_present:
			pre_scan_notPresent();			
			case sleeping:
			pre_scan_sleeping();
			break;
			case bedMode:
			pre_scan_inBed();
			break;
			case default:
			pre_scan_default();
			break;
			
		//check if the room is very hot, humid or has a high co2 ppm
		//send an alarm if so.
		checkExtremes();
			
			
		}
			
	
	}
	
}

void MainState::updateState_fromSleeping(){
	//recent activity out of bed
	if(recent(movement[m_BEDLEFT]) || recent(movement[m_BEDRIGHT])){
		*userState = bedMode; //make enum for exclusive states
		bedMode_init(); 
	}
}

//only movementsensor at the door directly relevant.
void MainState::pre_scan_notPresent(){
	
	updateState_fromNotPresent();
}				

//only movementsensor around bed and at door is relevant. Sound
//an alarm if the sensor at the door is activated in this state
//leave the sleeping state as soon as the bed side sensors are
//activated
void MainState::pre_scan_sleeping(){
	
	updateState_fromSleeping();
}

//for example always started after sleeping. Check if the user 
//wants to sleep (not yet alarm time.) 
void MainState::pre_scan_bedMode(){
	
	lampCheck_outOfBed();
	lampCheck_Bathroom();
	
	updateState_fromBedMode();
}

//default scan mode, controls lights based on light level and
//movement
void MainState::pre_scan_default(){

	if(*lightValues_updated){
		def_lampcheck_Door();
		def_lampCheck_Kitchen();
		def_lampCheck_Bureau();
		def_lampCheck_CeilingAndRadiator();
	}
	else{lampCheck_Kitchen; }
	
	lampCheck_Bathroom();
	updateState_fromDefault();
}


void MainState::update_music(){}

void MainState::update_computer(){}
//end of determining functions




//inline functions present for more readable code
inline void MainState::def_lampcheck_Door(){	
	if(lightValues[l_DOOR] < 300	&& !lampOn[l_DOOR]){				
		std::cout<<"turning lamp at door on\n"; //add function turn lamps off
	}
	else if(lightValues[l_DOOR] > 300+50	&& lampOn[l_DOOR]){
		std::cout<<"turning lamp at door off\n"; //add function turn lamps off	
	}
}

inline void MainState::def_lampCheck_Kitchen(){	
	if(lightValues[l_KITCHEN] < 300	&& !lampOn[l_KITCHEN] && recent(movement[m_KITCHEN])){
		std::cout<<"turning kitchen lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_KITCHEN] > 300+50	&& lampOn[l_KITCHEN] && !recent(movement[m_KITCHEN])){
		std::cout<<"turning lamp at kitchen off\n"; 		//function turn lamps off	
	}
}

inline void MainState::def_lampCheck_CeilingAndRadiator(){
	if(lightValues[l_BED] < 300 && (!lampOn[l_CEILING] || !lampOn[l_RADIATOR])){
		std::cout<<"turning heater ceiling lamps on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_BED] < 300+50 && (lampOn[l_CEILING] || lampOn[l_RADIATOR])){
		std::cout<<"turning heater ceiling lamp off\n"; 		//function turn lamps off	
	}
}

inline void MainState::def_lampCheck_Bureau(){
	if(lightValues[l_BED] < 300 && !lampOn[l_RADIATOR]){
		std::cout<<"turning bureau lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_BED] < 300+50 && (lampOn[l_CEILING] || lampOn[l_RADIATOR])){
		std::cout<<"turning lamp at bureau off\n"; 		//function turn lamps off	
	}	
}

inline void MainState::lampCheck_Bathroom(){
	if(recent(movement[m_BATHROOM]) && !lampOn[l_BATHROOM]){
		std::cout<<"turning bathroom lamp on\n"; 		//function turn lamps off
	}
	else if(lampOn[l_BATHROOM] && !recent(movement[m_BATHROOM])){
		std::cout<<"turning bathroom lamp off\n"; 		//function turn lamps off	
	}	
}

inline void lampCheck_outOfBed(){
	if(recent(movement[m_BEDLEFT]) || recent(movement[m_BEDRIGHT])){
		if(movement[m_BEDLEFT]-lastBedMovement)
		
		std::cout<<"turning radiator lamp on\n";
	}
	
	
}
