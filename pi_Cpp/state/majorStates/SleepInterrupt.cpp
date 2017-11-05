#include "SleepInterrupt.h"

SleepInterrupt::SleepInterrupt(StateData* stateData)
	: State(stateData)
{
	data->stateName = SLEEPINTERRUPT_S;
	std::cout<<"Ran SleepInterrupt state constructor"<<"\n";
}

SleepInterrupt::~SleepInterrupt(){
	std::cout<<"cleaned up the SleepInterrupt state"<<"\n";
}

bool SleepInterrupt::stillValid(){
	std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void SleepInterrupt::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";
}



