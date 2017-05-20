#ifndef WAKE
#define WAKE

#include "../mainState.h"

class Wakeup : public State
{

	public:
		Wakeup(StateData &stateData);
		~Wakeup();
		bool stillValid();
		void updateOnSensors();
	
	private:
		//std::thread lampThread;
		//std::atomic<bool> notShuttingDown;

		//void lampManagment(Wakeup* wakeup);
};

#endif
