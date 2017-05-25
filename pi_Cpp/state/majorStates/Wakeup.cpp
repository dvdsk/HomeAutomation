#include "Wakeup.h"

WakeUp::WakeUp(StateData* stateData)
	: State(stateData)
{
	stateName = WAKEUP_S;

#ifndef NOTHREAD
	stop = false;
	m_thread = new std::thread(threadFunction, this);
#endif	

	std::cout<<"Ran Wakeup state constructor\n";
	std::cout<<"stateName: "<<stateName<<"\n";
}

WakeUp::~WakeUp(){
	
#ifndef NOTHREAD
	stop = true;
	std::cout<<"send stop signal\n";
	m_thread->join();
#endif	

	std::cout<<"cleaned up the Wakeup state\n";
}

bool WakeUp::stillValid(){
	std::cout<<"decided its still the right state\n";
	return true;
}

void WakeUp::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff\n";
}
