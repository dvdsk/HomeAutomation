#include "Minimal.h"

Minimal::Minimal(StateData* stateData)
	: State(stateData)
{
	stateName = MINIMAL_S;
	std::cout<<"Ran Minimal state constructor"<<"\n";
}

Minimal::~Minimal(){
	std::cout<<"cleaned up the Minimal state"<<"\n";
}

bool Minimal::stillValid(){
	//std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void Minimal::updateOnSensors(){
	//std::cout<<"updated based on sensor values and stuff"<<"\n";
}



