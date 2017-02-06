#include "mainState.h"

void MainState::init_almostSleeping(MajorStates fromState){
	
	if(fromState == sleeping){
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

