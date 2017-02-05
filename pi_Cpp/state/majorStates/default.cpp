#include "mainState.h"

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

void MainState::environmental_alarm(){
	
}

//inline functions present for more readable code
void MainState::def_lampcheck_Door(){	
	if(lightValues[l_DOOR] < 300	&& !lampOn[l_DOOR]){				
		std::cout<<"turning lamp at door on\n"; //add function turn lamps off
	}
	else if(lightValues[l_DOOR] > 300+50	&& lampOn[l_DOOR]){
		std::cout<<"turning lamp at door off\n"; //add function turn lamps off	
	}
}

void MainState::def_lampCheck_Kitchen(){	
	if(lightValues[l_KITCHEN] < 300	&& !lampOn[l_KITCHEN] && recent(movement[m_KITCHEN])){
		std::cout<<"turning kitchen lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_KITCHEN] > 300+50	&& lampOn[l_KITCHEN] && !recent(movement[m_KITCHEN])){
		std::cout<<"turning lamp at kitchen off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_CeilingAndRadiator(){
	if(lightValues[l_BED] < 300 && (!lampOn[l_CEILING] || !lampOn[l_RADIATOR])){
		std::cout<<"turning heater ceiling lamps on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_BED] < 300+50 && (lampOn[l_CEILING] || lampOn[l_RADIATOR])){
		std::cout<<"turning heater ceiling lamp off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_Bureau(){
	if(lightValues[l_BED] < 300 && !lampOn[l_RADIATOR]){
		std::cout<<"turning bureau lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[l_BED] < 300+50 && (lampOn[l_CEILING] || lampOn[l_RADIATOR])){
		std::cout<<"turning lamp at bureau off\n"; 		//function turn lamps off	
	}	
}

void MainState::lampCheck_Bathroom(){
	if(recent(movement[m_BATHROOM]) && !lampOn[l_BATHROOM]){
		std::cout<<"turning bathroom lamp on\n"; 		//function turn lamps off
	}
	else if(lampOn[l_BATHROOM] && !recent(movement[m_BATHROOM])){
		std::cout<<"turning bathroom lamp off\n"; 		//function turn lamps off	
	}	
}
