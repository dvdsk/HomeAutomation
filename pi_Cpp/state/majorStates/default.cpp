#include "mainState.h"

void MainState::init_default(){
	majorState = DEFAULT;
	
}

void MainState::transitions_default(){
	if(!minorState.showering && anyRecent(movement)){
		init_away();
	}
}

void MainState::update_default(){

	if(*lightValues_updated){
		def_lampcheck_Door();
		def_lampCheck_Kitchen();
		def_lampCheck_Bureau();
		def_lampCheck_CeilingAndRadiator();
	}
	else{lampCheck_Kitchen; }
	lampCheck_Bathroom();
	
	environmental_alarm();
	check_Plants();
	transitions_default();
}

void MainState::environmental_alarm(){
	for(temp : tempValues)
		if(temp > config::ALERT_TEMP_ABOVE){
			if(temp > config::ALARM_TEMP_ABOVE){
				//full alarm
			}
			else{
				//text alart
			}
		}
	for(humidity : humidityValues)
		if(humidity > config::ALERT_HUMIDITY_ABOVE){
			if(humidity > config::ALARM_HUMIDITY_ABOVE){
				//full alarm
			}
			else{
				//text alart				
			}
		}
	if(CO2ppm > ALERT_CO2PPM){
		if(CO2ppm > ALARM_CO2PPM){
				//full alarm			
		}
		else{
				//text alart					
		}
	}
}



//functions
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
