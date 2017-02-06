#include "mainState.h"

void MainState::update_sleeping(){
	if(recent(movement[mov::DOOR])){
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
	if(recent(movement[m_BEDLEFT]) || recent(movement[m_BEDRIGHT])){
		majorState = ALMOSTSLEEPING;
		init_almostSleeping(SLEEPING); 
	}
	return;
}
