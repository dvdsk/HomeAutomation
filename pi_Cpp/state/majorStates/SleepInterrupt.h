#ifndef SLEEPINTTERUPT
#define SLEEPINTTERUPT

#include "../mainState.h"

class SleepInterrupt : public State
{

	public:
		SleepInterrupt(StateData &stateData);
		~SleepInterrupt();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
