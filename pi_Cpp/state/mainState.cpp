#include "mainState.h"

//decode url to a command to change a state or pass to a function
void MainState::httpSwitcher(const char* raw_url){

	if(0 == strcmp(raw_url, "/lamps")){
		std::cout<<"lamps call has been made\n";
		parseCommand(LIGHTS_ALLOFF);
	}
	if(0 == strcmp(raw_url, "/sleep")){
		std::cout<<"sleep call has been made\n";
		majorState = SLEEPING;
	}
	return;
}

void MainState::parseCommand(Command toParse){
	
	switch(toParse){
		case LIGHTS_ALLON:
		std::cout<<"all lights are now on";
		break;
		
		case LIGHTS_ALLOFF:
		std::cout<<"all lights are now off";		
		break;
		
		case MS_SLEEPING:
		majorState = SLEEPING;
		runUpdate();
		break;
		
		case MOVIEMODE:
		minorState.movieMode = true;
		break;
		
	}
}

MainState::MainState(){
	is_ready = false;
}

void MainState::thread_watchForUpdate(){
	std::unique_lock<std::mutex> lk(m);
	
	while(true){ //can later be replaced with mutex to check if we should stop
		cv.wait(lk);
		currentTime = (uint32_t)time(nullptr);
		
		switch(majorState){
			case AWAY:
			update_away();
			break;			
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

/////////////////////////////////////////////////////////////////////////////////////////////////////

void MainState::init_away(){
	
	majorState = AWAY;
	
	//turn all lamps off
	//turn music off
	//if computer on ask if computer may be turned off via telegram
}

void MainState::transitions_away(){
	if(recent(movement[mov::DOOR], 5)){
		if(minorState.authorisedClose){
			init_default();
		}
		else{
			alarmProcedureStarted.lock();
			away_intruder_alarm();
		}
	}
	return;
}

void MainState::update_away(){
	
	check_Plants();
	transitions_away();
	return;
}				

void MainState::away_intruder_alarm(){
	minorState.alarmDisarm = false;
	
	//send telegram message
	//start beep (to indicate alarm still armed)
	sleep(30);	
	//message on all possible channals
	//message other people
	
	while(!minorState.authorisedClose || !minorState.alarmDisarm){
		//flash lamps, beeb, scream intruder
		//do horrible stuff to scare of burgalars		
	}
	alarmProcedureStarted.unlock();
	return;
}

void MainState::check_Plants(){
	char* warningText = (char*)"SOIL HUMIDITY ALERT\n\n";
	bool sendAlert = false;
	for(int i = 0; i<plnt::NUMB_OF_PLANT_SENSORS; i++){
		if(soilHumidityValues[i] < plnt::ALERT_HUMIDITY_BELOW[i])		
			warningText = strcat(warningText, plnt::NAMES[i]);
			warningText = strcat(warningText, "'s humidity has dropped below the minimum\n");
			sendAlert = true;
	}
	if(sendAlert){
		//send a telegram message		
	}
	return;
}

/////////////////////////////////////////////////////////////////////////////////////

void MainState::update_sleeping(){
	if(recent(movement[mov::DOOR], 5)){
		alarmProcedureStarted.lock();
		night_alarm();
	}
	transitions_sleeping();
	return;
}

void MainState::night_alarm(){
	minorState.alarmDisarm = false;
	while(!minorState.alarmDisarm){
		//light weakly on
		//alarm sound on growing louder
		//send out warnings (with is ok button)
	}
	return;
}

void MainState::init_sleeping(){
	
	//turn off all lights
	//turn off all audio
	
	//if computer on ask turn off (by bedbutton?/ sound?)
	return;
}

void MainState::transitions_sleeping(){

	//recent activity out of bed
	//TODO make failsafe.
	if(recent(movement[mov::BED_l], 2) || recent(movement[mov::BED_r], 2)){
		majorState = ALMOSTSLEEPING;
		init_almostSleeping(SLEEPING); 
	}
	return;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////


void MainState::init_default(){
	majorState = DEFAULT;
	
}

void MainState::transitions_default(){
	if(!minorState.showering && anyRecent(movement, 600)){
		init_away();
	}
}

void MainState::update_default(){

	if(lightValues_updated){
		def_lampcheck_Door();
		def_lampCheck_Kitchen();
		def_lampCheck_Bureau();
		def_lampCheck_CeilingAndRadiator();
	}
	else{def_lampCheck_Kitchen(); }
	lampCheck_Bathroom();
	
	environmental_alarm();
	check_Plants();
	transitions_default();
}

void MainState::environmental_alarm(){
	for(auto temp : tempValues)
		if(temp > config::ALERT_TEMP_ABOVE){
			if(temp > config::ALARM_TEMP_ABOVE){
				//full alarm
			}
			else{
				//text alart
			}
		}
	for(auto humidity : humidityValues)
		if(humidity > config::ALERT_HUMIDITY_ABOVE){
			if(humidity > config::ALARM_HUMIDITY_ABOVE){
				//full alarm
			}
			else{
				//text alart				
			}
		}
	if(CO2ppm > config::ALERT_CO2PPM){
		if(CO2ppm > config::ALARM_CO2PPM){
				//full alarm			
		}
		else{
				//text alart					
		}
	}
}



//functions
void MainState::def_lampcheck_Door(){	
	if(lightValues[lght::DOOR] < 300	&& !lampOn[lmp::DOOR]){				
		std::cout<<"turning lamp at door on\n"; //add function turn lamps off
	}
	else if(lightValues[lght::DOOR] > 300+50	&& lampOn[lmp::DOOR]){
		std::cout<<"turning lamp at door off\n"; //add function turn lamps off	
	}
}

void MainState::def_lampCheck_Kitchen(){	
	if(lightValues[lght::KITCHEN] < 300	&& !lampOn[lmp::KITCHEN] 
	&& recent(movement[mov::KITCHEN], config::KTCHN_TIMEOUT)){
		std::cout<<"turning kitchen lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::KITCHEN] > 300+50	&& lampOn[lmp::KITCHEN] 
	&& !recent(movement[mov::KITCHEN], config::KTCHN_TIMEOUT)){
		std::cout<<"turning lamp at kitchen off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_CeilingAndRadiator(){
	if(lightValues[lght::BED] < 300 
	&& (!lampOn[lmp::CEILING] || !lampOn[lmp::RADIATOR])){
		std::cout<<"turning heater ceiling lamps on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::BED] < 300+50 
	&& (lampOn[lmp::CEILING] || lampOn[lmp::RADIATOR])){
		std::cout<<"turning heater ceiling lamp off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_Bureau(){
	if(lightValues[lght::BED] < 300 && !lampOn[lmp::RADIATOR]){
		std::cout<<"turning bureau lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::BED] < 300+50 
	&& (lampOn[lmp::CEILING] || lampOn[lmp::RADIATOR])){
		std::cout<<"turning lamp at bureau off\n"; 		//function turn lamps off	
	}	
}

void MainState::lampCheck_Bathroom(){
	if(recent(movement[mov::BATHROOM], config::WCPIR_TIMEOUT) 
	&& !lampOn[lmp::BATHROOM]){
		std::cout<<"turning bathroom lamp on\n"; 		//function turn lamps off
	}
	else if(lampOn[lmp::BATHROOM] 
	&& !recent(movement[mov::BATHROOM], config::WCPIR_TIMEOUT)){
		std::cout<<"turning bathroom lamp off\n"; 		//function turn lamps off	
	}	
}

/////////////////////////////////////////////////////////////////////////////////////////////////////


void MainState::init_almostSleeping(MajorStates fromState){
	
	if(fromState == SLEEPING){
		//toilet or fridge/water sleep break thus:		
		
		//lamp color and brightness to night red
		//lights on towards toilet
	}
	else{
		//lamp color and brightness to night red
	}
	
	//volume to sleep level
}


void MainState::transitions_almostSleeping(){
	//no automatic recovery from this state.
}


//for example always started after sleeping. Check if the user 
//wants to sleep (not yet alarm time.) 
void MainState::update_almostSleeping(){
	
	lampCheck_Bathroom();
	
	transitions_almostSleeping();
}

//GENERAL FUNCTIONS
/////////////////////////////////////////////////////////////////////////////////////////////////////

inline void sleep(int seconds){
	std::this_thread::sleep_for(std::chrono::seconds(seconds));
}

inline bool MainState::recent(uint32_t time, unsigned int threshold){
	if(currentTime-time > threshold){return true; }
	else{return false; }
}

inline bool MainState::anyRecent(std::array<uint32_t, 5> times,
unsigned int threshold){
	bool recent = false;
	for(auto time : times)
		if(currentTime - time < threshold){recent = true; } 
	return recent;
}

void MainState::runUpdate(){
	std::unique_lock<std::mutex> lk(m);
	is_ready = true;
	cv.notify_one();
}
