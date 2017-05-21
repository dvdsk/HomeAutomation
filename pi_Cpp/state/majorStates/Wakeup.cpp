#include "Wakeup.h"

//TODO try this: http://stackoverflow.com/questions/5956759/c11-stdthread-inside-a-class-executing-a-function-member-with-thread-initia

Wakeup::Wakeup(StateData &stateData)
	: State(&stateData){
	stateName = WAKEUP_S;	

  //m_thread = new std::thread(threadFunction,this);

	std::cout<<"Ran Wakeup state constructor"<<"\n";
	std::cout<<"StateName: "<<stateName<<"\n";
	std::cout<<"Wakeup_s: "<<WAKEUP_S<<"\n";
}

Wakeup::~Wakeup(){

	std::cout<<"shutting down wakeup state\n";
	//notShuttingDown = false;
	//m_thread->join();
	//try {stop();} catch(std::system_error){std::cout<<"caught error";}
	std::cout<<"cleaned up the Wakeup state"<<"\n";
}

bool Wakeup::stillValid(){
	std::cout<<"decided its still the right state\n";

	//if user has been in kitchen area

	return true;
}

void Wakeup::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";

	//disarm night alarm if activity near bed
	//bathroom light on if movement
}

//void Wakeup::lampManagment(Wakeup* wakeup){
//	
//	while(wakeup->notShuttingDown){}
//		

//	std::cout<<"killing lamp managment\n";
//}

