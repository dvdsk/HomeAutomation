#include "mainState.h"

void MainState::init_away(){
	
	majorState = AWAY;
	
	//turn all lamps off
	//turn music off
	//if computer on ask if computer may be turned off via telegram
}

void MainState::transitions_away(){
	if(recent(movement[mov::DOOR])){
		if(authorisedClose){
			init_default();
		}
		else{
			alarmProcedureStarted.lock();
			intruder_alarm();
		}
	}
	return;
}

void MainState::update_away(){
	
	
	
	transitions_away();
	return;
}				

void MainState::away_intruder_alarm(){
	//send telegram message
	//start beep (to indicate alarm still armed)
	sleep(30);	
	//message on all possible channals
	//message other people
	
	while(!authorised){
		//flash lamps, beeb, scream intruder
		//do horrible stuff to scare of burgalars		
	}
	return;
}


inline void sleep(constexpr int seconds){
	thrd_sleep(&(struct timespec){.tv_sec=seconds}, NULL);
}
