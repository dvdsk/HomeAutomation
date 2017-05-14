#ifndef MINIMAL
#define MINIMAL

#include "../mainState.h"

class Minimal : public State
{

	public:
		Minimal(StateData &stateData);
		~Minimal();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
