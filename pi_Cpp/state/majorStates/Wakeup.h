#ifndef WAKEUP_G
#define WAKEUP_G

#include "../mainState.h"

static void threadFunction() {
	std::cout<<"helloa how are you?\n";
	return;
}

class WakeUp : public State
{

	public:
		WakeUp(StateData &stateData);
		~WakeUp();
		bool stillValid();
		void updateOnSensors();
	
	private:

#ifndef NOTHREAD
	std::thread* m_thread;
#endif	
};

#endif
