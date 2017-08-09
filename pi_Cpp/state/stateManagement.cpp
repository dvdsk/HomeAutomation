#include "mainState.h"
#include "majorStates/default.h"
//#include "majorStates/GoingToSleep.h"
//#include "majorStates/SleepInterrupt.h"
#include "majorStates/Minimal.h"
#include "majorStates/Wakeup.h"

inline void startNewState(State* &currentState, StateData* stateData){
	switch(stateData->newState){
//		case AWAY:
//		//currentState = new Default();
//		break;			
//		case SLEEPING:
//		break;
		case DEFAULT_S:
		currentState = new Default(stateData);
		break;
//		case GOINGTOSLEEP_S:
//		currentState = new GoingToSleep(&stateData);
//		break;
//		case SLEEPINTERRUPT_S:
//		currentState = new SleepInterrupt(&stateData);
//		break;
		case MINIMAL_S:
		currentState = new Minimal(stateData);
		break;
		case WAKEUP_S:
		currentState = new WakeUp(stateData);
		break;
	}
}

void thread_state_management(std::atomic<bool>* notShuttingdown,
	   StateData* stateData, SignalState* signalState){

	State* currentState = new Default(stateData);

	std::unique_lock<std::mutex> lk(signalState->m);
	while(*notShuttingdown){
		signalState->cv.wait(lk, [signalState]{return signalState->signalled;});//wait for new sensor data or forced update.	

		stateData->currentTime = (uint32_t)time(nullptr);
		if(currentState->stillValid()) //TODO FIXME 
			currentState->updateOnSensors();
		else{
			delete currentState;
			startNewState(currentState, stateData);
		}
		if(stateData->httpState->updated){
			if(currentState->updateOnHttp()){
				//updateOnHttp returns true if new state needs to be started
				delete currentState;
				startNewState(currentState, stateData);
			}				
		}
	}
	delete currentState;
	delete stateData;
}
