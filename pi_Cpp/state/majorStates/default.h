#ifndef DEFAULT
#define DEFAULT

#include "../mainState.h"

class Default : public State
{

	public:
		Default(StateData* stateData, int* testInt);
		~Default();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
