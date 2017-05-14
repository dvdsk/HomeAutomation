#ifndef DEFAULT
#define DEFAULT

#include "../mainState.h"

class Default : public State
{

	public:
		Default(StateData &stateData);
		~Default();
		bool stillValid();
		void updateOnSensors();
	
	private:
		


};

#endif
