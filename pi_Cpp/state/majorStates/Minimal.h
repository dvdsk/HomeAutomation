#ifndef MINIMAL_G
#define MINIMAL_G

#include "../mainState.h"

class Minimal : public State
{

	public:
		Minimal(StateData* stateData, int* testInt);
		~Minimal();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
