#include "Wakeup.h"

WakeUp::WakeUp(StateData &stateData)
	: State(&stateData)
{
	stateName = WAKEUP_S;
	std::cout<<"Ran Wakeup state constructor"<<"\n";
}

WakeUp::~WakeUp(){
	std::cout<<"cleaned up the Wakeup state"<<"\n";
}

bool WakeUp::stillValid(){
	std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void WakeUp::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";
}

