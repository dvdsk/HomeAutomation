#include "mainState.h"

inline void lampCheck_outOfBed(){
	if(recent(movement[m_BEDLEFT]) || recent(movement[m_BEDRIGHT])){
		if(movement[m_BEDLEFT]-lastBedMovement)
		
		std::cout<<"turning radiator lamp on\n";
	}
	
	
}
//for example always started after sleeping. Check if the user 
//wants to sleep (not yet alarm time.) 
void MainState::pre_scan_bedMode(){
	
	lampCheck_outOfBed();
	lampCheck_Bathroom();
	
	updateState_fromBedMode();
}

