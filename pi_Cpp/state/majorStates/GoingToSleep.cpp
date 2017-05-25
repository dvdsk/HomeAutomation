#include "GoingToSleep.h"

GoingToSleep::GoingToSleep(StateData* stateData)
	: State(stateData)
{
	stateName = GOINGTOSLEEP_S;	
	std::cout<<"Ran GoingToSleep state constructor"<<"\n";
}

GoingToSleep::~GoingToSleep(){
	std::cout<<"cleaned up the GoingToSleep state"<<"\n";
}

bool GoingToSleep::stillValid(){
	std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void GoingToSleep::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";
}



