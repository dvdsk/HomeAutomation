#include "default.h"

void Default::Default(StateData &stateData){
	std::cout<<"Ran default state constructor<<\n"
}

void ~Default::Default(){
	std::cout<<"cleaned up the default state<<\n"
}

bool Default::stillValid(){
	std::cout<<"decided its still the right state<<\n"
	return true;
}

void Default::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff<<\n"
}



