#include "default.h"

using namespace std::chrono_literals;

std::condition_variable cv_default;
std::mutex cv_default_m;

static void lightColor_thread(Default* currentState){
  std::unique_lock<std::mutex> lk(cv_default_m);	

	while(!currentState->stop.load()){
		std::cout<<"updating light color\n";
		
	//lamps->setState("{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");

	
		cv_default.wait_for(lk, 5*1s, [currentState](){return currentState->stop.load();});
	}
}				

Default::Default(StateData* stateData)
	: State(stateData){
	stateName = DEFAULT_S;	

	stop = false;
	m_thread = new std::thread(lightColor_thread, this);

	
	std::cout<<"Ran default state constructor"<<"\n";
}

Default::~Default(){
	stop = true;
	cv_default.notify_all();
	m_thread->join();

	std::cout<<"cleaned up the default state"<<"\n";
}

bool Default::stillValid(){
	std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void Default::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff"<<"\n";
}



