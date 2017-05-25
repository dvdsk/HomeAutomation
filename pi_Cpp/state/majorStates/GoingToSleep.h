#ifndef GOINGTOSLEEP
#define GOINGTOSLEEP

#include "../mainState.h"

class GoingToSleep : public State
{

	public:
		GoingToSleep(StateData* stateData);
		~GoingToSleep();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
