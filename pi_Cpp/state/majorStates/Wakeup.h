#ifndef WAKEUP_G
#define WAKEUP_G

#include "../mainState.h"

class WakeUp : public State
{

	public:
		WakeUp(StateData &stateData);
		~WakeUp();
		bool stillValid();
		void updateOnSensors();
	
	private:

};

#endif
