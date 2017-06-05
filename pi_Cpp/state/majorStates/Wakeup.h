#ifndef WAKEUP_G
#define WAKEUP_G

#include "../mainState.h"



class WakeUp : public State
{

	public:
		WakeUp(StateData* stateData);
		~WakeUp();
		bool stillValid();
		void updateOnSensors();
	
	private:
		std::atomic<bool> stop;
		static void* threadFunction(WakeUp* arg) {
			std::cout<<"by\n";
			return 0;
		}

	
		std::thread* m_thread;
};

#endif
