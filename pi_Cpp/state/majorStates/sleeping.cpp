#include "mainState.h"

void MainState::updateState_fromSleeping(){
	//recent activity out of bed
	if(recent(movement[m_BEDLEFT]) || recent(movement[m_BEDRIGHT])){
		*userState = bedMode; //make enum for exclusive states
		bedMode_init(); 
	}
}

//only movementsensor around bed and at door is relevant. Sound
//an alarm if the sensor at the door is activated in this state
//leave the sleeping state as soon as the bed side sensors are
//activated
void MainState::pre_scan_sleeping(){
	
	updateState_fromSleeping();
}
