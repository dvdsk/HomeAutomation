#include "default.h"

Default::Default(StateData* stateData)
	: State(stateData){
	stateName = DEFAULT_S;	
	std::cout<<"Ran default state constructor"<<"\n";
}

Default::~Default(){
	//FIXME This frees memory needed by stateData???? why???
	std::cout<<"cleaned up the default state"<<"\n";
}

bool Default::stillValid(){
	std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void Default::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";
}



