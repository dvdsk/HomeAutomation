#include "Wakeup.h"

using namespace std::chrono_literals;

std::condition_variable cv;
std::mutex cv_m;

static void* threadFunction(WakeUp* currentState){				

  std::unique_lock<std::mutex> lk(cv_m);

	std::cout<<"waiting\n";

	//wait for 20 seconds or till the cv is notified and stop is set to true
	cv.wait_for(lk, 200*100ms, [currentState](){return currentState->stop.load();});


	std::cout<<"done\n";
	return 0;
}

WakeUp::WakeUp(StateData* stateData)
	: State(stateData)
{
	stateName = WAKEUP_S;

	stop = false;
	m_thread = new std::thread(threadFunction, this);

	std::cout<<"Ran Wakeup state constructor\n";
	std::cout<<"stateName: "<<stateName<<"\n";
}

WakeUp::~WakeUp(){
	
	stop = true;
	cv.notify_all();
	std::cout<<"send stop signal\n";
	m_thread->join();

	std::cout<<"cleaned up the Wakeup state\n";
}

bool WakeUp::stillValid(){
	std::cout<<"decided its still the right state\n";
	return true;
}

void WakeUp::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff\n";
}
