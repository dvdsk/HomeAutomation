#include "mainState.h"
#include "majorStates/default.h"
//#include "majorStates/GoingToSleep.h"
//#include "majorStates/SleepInterrupt.h"
#include "majorStates/Minimal.h"
#include "majorStates/Wakeup.h"

inline void startNewState(State* currentState, StateData* stateData, int* testInt){
	switch(stateData->newState){
//		case AWAY:
//		//currentState = new Default();
//		break;			
//		case SLEEPING:
//		break;
		case DEFAULT_S:
		currentState = new Default(stateData, testInt);
		break;
//		case GOINGTOSLEEP_S:
//		currentState = new GoingToSleep(&stateData);
//		break;
//		case SLEEPINTERRUPT_S:
//		currentState = new SleepInterrupt(&stateData);
//		break;
		case MINIMAL_S:
		currentState = new Minimal(stateData, testInt);
		break;
		case WAKEUP_S:
		currentState = new WakeUp(stateData, testInt);
		break;
	}
}

void thread_state_management(std::shared_ptr<std::atomic<bool>> notShuttingdown,
	SignalState* signalState, SensorState* sensorState, MpdState* mpdState, 
	Mpd* mpd, HttpState* httpState, ComputerState* computerState){

	//StateData stateData(sensorState, mpdState, mpd, httpState, computerState);
	StateData* stateData = new StateData(sensorState, mpdState, mpd, httpState, computerState);

	int* testInt = new int;
	*testInt = 42;

	State* currentState = new Default(stateData, testInt);
 
	computerState->off = true;
	
	std::unique_lock<std::mutex> lk(signalState->m);
	while(*notShuttingdown){
		signalState->cv.wait(lk);//wait for new sensor data or forced update.
		std::cout<<"running update\n";		

		stateData->currentTime = (uint32_t)time(nullptr);
//		if(currentState->stillValid()) //TODO FIXME 
//			currentState->updateOnSensors();
//		else{
//			delete currentState;
//			startNewState(currentState, stateData);
//		}

		std::cout<<"hiii0\n";
		if(httpState->updated){
			std::cout<<"hiii1\n";
			if(currentState->updateOnHttp()){
				//updateOnHttp returns true if new state needs to be started
				std::cout<<"hiii2\n";
				delete currentState;
				startNewState(currentState, stateData, testInt);
			}				
		}
	}
	delete currentState;
	delete stateData;
}
