#include "mainState.h"

void thread_state_manager(std::shared_ptr<MainState> state, std::shared_ptr<Mpd> mpd,
     std::shared_ptr<std::atomic<bool>> notShuttingdown){
	state->watchForUpdate(mpd, notShuttingdown);
	std::cout<<"State menagement shut down gracefully\n";
	return;
}

//decode url to a command to change a state or pass to a function
void MainState::httpSwitcher(const char* raw_url){
	std::string url = raw_url;
	if(url == "lamps/evening"){
		if(majorState != MINIMAL){majorState = MINIMAL; signalUpdate(); }
		setState("{\"bri\":220, \"ct\":200, \"transitiontime\":10}");
	}
	if(url == "lamps/night"){
		if(majorState != MINIMAL){majorState = MINIMAL; signalUpdate(); }
		setState("{\"bri\":120, \"ct\":153, \"transitiontime\":10}");
	}
	if(url == "lamps/bedlight"){
		if(majorState != MINIMAL){majorState = MINIMAL; signalUpdate(); }
		setState("{\"bri\":1, \"ct\":153, \"transitiontime\":10}");
	}
	if(url == "lamps/normal"){
		if(majorState != MINIMAL){majorState = MINIMAL; signalUpdate(); }
		setState("{\"bri\":254, \"ct\":300, \"transitiontime\":10}");
	}

	if(url == "state/away"){
		if(majorState != AWAY){majorState = AWAY; signalUpdate(); }
	}
	if(url == "state/default"){
		if(majorState != DEFAULT){majorState = DEFAULT; signalUpdate(); }
	}
	if(url == "state/almostsleeping"){
		if(majorState != ALMOSTSLEEPING){majorState = ALMOSTSLEEPING; signalUpdate(); }
	}
	if(url == "state/sleeping"){
		if(majorState != SLEEPING){majorState = SLEEPING; signalUpdate(); }
	}
	if(url == "state/minimal"){
		if(majorState != MINIMAL){majorState = MINIMAL; signalUpdate(); }
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
		signalUpdate();
		break;
		
		case MOVIEMODE:
		minorState.movieMode = true;
		break;
		
	}
}

MainState::MainState()
	: Lamps(){
	//TODO do something to figure these out
	is_ready = false;
	majorState = DEFAULT;
	
	minorState.alarmDisarm = false;
	minorState.authorisedClose = false;
	minorState.listenToAudioBook = false;
	minorState.wakingUp = false;
  minorState.inBathroom = false;
  minorState.showering = false;
  minorState.inKitchenArea = false;
  minorState.movieMode = false;

	lightValues_updated = false;
	tempValues_updated = false;
	humidityValues_updated = false;
	soilHumidity_updated = false;
	CO2ppm_updated = false;
	movement[0] =0 ;
	
}

void MainState::watchForUpdate(std::shared_ptr<Mpd> mpd, 
	   std::shared_ptr<std::atomic<bool>> notShuttingdown){

	std::unique_lock<std::mutex> lk(m);
	
	while(*notShuttingdown){
		cv.wait(lk);
		//std::cout<<"running update\n";		

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
			case MINIMAL:
			update_minimal();
			break;
		}
	}
	return;
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
	
	environmental_alarm();
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
	std::string warningText = "SOIL HUMIDITY ALERT\n\n";
	bool sendAlert = false;
	int percentBelow;
	for(int i = 0; i<plnt::NUMB_OF_PLANT_SENSORS; i++){
		if(soilHumidityValues[i] < plnt::ALERT_HUMIDITY_BELOW[i])		
			percentBelow = (plnt::ALERT_HUMIDITY_BELOW[i] - soilHumidityValues[i])
										 /plnt::ALERT_HUMIDITY_BELOW[i]*100;
			warningText += std::string(plnt::NAMES[i]);
			warningText += "'s humidity ";
			warningText += std::to_string(percentBelow);
			warningText += "% below the minimum\n";
			sendAlert = true;
	}
	if(sendAlert){
		//std::cout<<"sending telegram message: \n"<<warningText;
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
	//TODO make update_Lamps();
	environmental_alarm();
	check_Plants();
	transitions_default();
}

void MainState::environmental_alarm(){
	//TODO IMPLEMENT LOWER BOUNDS
	std::string alertString = "ALERT!!:\n";
	std::string alarmString = "ALARM!!:\n";
	bool alert = false;
	bool alarm = false;
	
	//check temperature values TODO adjust TEMP_ABOVE to account for
	//outside weather from weather forecast	
	for(unsigned int i=0; i<temp::LEN; i++){
		if(tempValues[i] > config::ALERT_TEMP_ABOVE){
			if(tempValues[i] > config::ALARM_TEMP_ABOVE){
				alarm = true;
				alarmString += "temperature "+std::string(temp::NAMES[i]) 
			  + "at "+std::to_string(tempValues[i]/10)+"°C\n";
			}
			else{
			  alert = true;
			  alertString += "temperature "+std::string(temp::NAMES[i]) 
			  + "at "+std::to_string(tempValues[i]/10)+"°C\n";
			}
		}
		if(tempValues[i] > config::ALERT_TEMP_BELOW){
			if(tempValues[i] > config::ALARM_TEMP_BELOW){
				alarm = true;
				alarmString += "temperature "+std::string(temp::NAMES[i]) 
			  + "at "+std::to_string(tempValues[i]/10)+"°C\n";
			}
			else{
			  alert = true;
			  alertString += "temperature "+std::string(temp::NAMES[i]) 
			  + "at "+std::to_string(tempValues[i]/10)+"°C\n";
			}
		}		
	}
	//check humidity with seperate bathroom handeling to account for 
	//increased humidity due to showering
	for(unsigned int i=0; i< hum::LEN; i++){
		if(i == hum::BATHROOM && !recent(mov::BATHROOM, config::DT_HUMIDALARM_SHOWER)){
			if(humidityValues[i] > config::ALERT_HUMIDITY_ABOVE){
				if(humidityValues[i] > config::ALARM_HUMIDITY_ABOVE){
					alarm = true;
					alarmString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%, last bathroom "
					+ "activity was: "+ toTime(mov::BATHROOM-currentTime)+" ago\n";	
				}
				else{
					alert = true;
					alertString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%, last bathroom "
					+ "activity was: "+ toTime(mov::BATHROOM-currentTime)+" ago\n";	
				}	
			}
			if(humidityValues[i] > config::ALERT_HUMIDITY_BELOW){
				if(humidityValues[i] > config::ALARM_HUMIDITY_BELOW){
					alarm = true;
					alarmString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";	
				}
				else{
					alert = true;
					alertString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";	
				}	
			}					
		}
		else{
			if(humidityValues[i] > config::ALERT_HUMIDITY_ABOVE){
				if(humidityValues[i] > config::ALARM_HUMIDITY_ABOVE){
					alarm = true;
					alarmString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";
				}
				else{
					alert = true;
					alertString	+= "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";		
				}
			}
			if(humidityValues[i] > config::ALERT_HUMIDITY_BELOW){
				if(humidityValues[i] > config::ALARM_HUMIDITY_BELOW){
					alarm = true;
					alarmString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";	
				}
				else{
					alert = true;
					alertString += "humidity "+std::string(hum::NAMES[i]) 
					+ "at "+std::to_string(humidityValues[i]/10)+"%\n";	
				}	
			}	
		}
	}
	//check co2 levels 
	if(CO2ppm > config::ALERT_CO2PPM){
		if(CO2ppm > config::ALARM_CO2PPM){
			alarm = true;
			alarmString += ": room co2 level at: "+std::to_string(CO2ppm)+" ppm\n";			
		}
		else{
			alert = true;
			alertString += ": room co2 level at: "+std::to_string(CO2ppm)+" ppm\n";						
		}		
	}
	
	if(alarm){}//std::cout<<alarmString; }
	if(alert){}//std::cout<<alertString; }
	
	return;
}



//functions
void MainState::def_lampcheck_Door(){	
	if(lightValues[lght::DOOR] < 300	&& !lampOn[lmp::DOOR]){				
		//std::cout<<"turning lamp at door on\n"; //add function turn lamps off
	}
	else if(lightValues[lght::DOOR] > 300+50	&& lampOn[lmp::DOOR]){
		//std::cout<<"turning lamp at door off\n"; //add function turn lamps off	
	}
}

void MainState::def_lampCheck_Kitchen(){	
	if(lightValues[lght::KITCHEN] < 300	&& !lampOn[lmp::KITCHEN] 
	&& recent(movement[mov::KITCHEN], config::KTCHN_TIMEOUT)){
		//std::cout<<"turning kitchen lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::KITCHEN] > 300+50	&& lampOn[lmp::KITCHEN] 
	&& !recent(movement[mov::KITCHEN], config::KTCHN_TIMEOUT)){
		//std::cout<<"turning lamp at kitchen off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_CeilingAndRadiator(){
	if(lightValues[lght::BED] < 300 
	&& (!lampOn[lmp::CEILING] || !lampOn[lmp::RADIATOR])){
		//std::cout<<"turning heater ceiling lamps on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::BED] < 300+50 
	&& (lampOn[lmp::CEILING] || lampOn[lmp::RADIATOR])){
		//std::cout<<"turning heater ceiling lamp off\n"; 		//function turn lamps off	
	}
}

void MainState::def_lampCheck_Bureau(){
	if(lightValues[lght::BED] < 300 && !lampOn[lmp::RADIATOR]){
		//std::cout<<"turning bureau lamp on\n"; 		//function turn lamps off
	}
	else if(lightValues[lght::BED] < 300+50 
	&& (lampOn[lmp::CEILING] || lampOn[lmp::RADIATOR])){
		//std::cout<<"turning lamp at bureau off\n"; 		//function turn lamps off	
	}	
}

void MainState::lampCheck_Bathroom(){
	if(recent(movement[mov::BATHROOM], config::WCPIR_TIMEOUT) 
	&& !lampOn[lmp::BATHROOM]){
		//std::cout<<"turning bathroom lamp on\n"; 		//function turn lamps off
	}
	else if(lampOn[lmp::BATHROOM] 
	&& !recent(movement[mov::BATHROOM], config::WCPIR_TIMEOUT)){
		//std::cout<<"turning bathroom lamp off\n"; 		//function turn lamps off	
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
	//TODO make this update_Lamps();
	lampCheck_Bathroom();
	transitions_almostSleeping();
}

/////////////////////////////////////////////////////////////////////////////////////
void MainState::init_minimal(MajorStates fromState){
	lastState = majorState;
	timeMinimalStarted = currentTime;
}

void MainState::update_minimal(){

	lampCheck_Bathroom();
	environmental_alarm();
	check_Plants();
	transitions_minimal();
}

void MainState::transitions_minimal(){
	if(timeMinimalStarted - currentTime < stateConf::MAXMINIMALDURATION){
	majorState = lastState;
	lastState = MINIMAL;	
	}
	return;
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

inline bool MainState::anyRecent(uint32_t times[],
unsigned int threshold){
	bool recent = false;
	for(int i=0; i<mainState::LEN_movement; i++)
		if(currentTime - times[i] < threshold){recent = true; } 
	return recent;
}

void MainState::signalUpdate(){
	//std::cout<<"signal to update\n";
	
	std::unique_lock<std::mutex> lk(m);
	is_ready = true;
	cv.notify_one();
}

inline std::string toTime(uint32_t seconds){
	if(seconds<3600){return std::to_string(seconds/60)+"minutes"; }
	else if(seconds<24*3600){return std::to_string(seconds/3600)+"hours"; }
	else{return std::to_string(seconds/(24*3600))+"days"; }
} 

inline bool setting_up_values_done(){

//	return( lightValues && tempValues_updated && humidityValues_updated && soilHumidity_updated && CO2ppm_updated && movement[0]>0 );
	return true; //TODO what is this for?

}
