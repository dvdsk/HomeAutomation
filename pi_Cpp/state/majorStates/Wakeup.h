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
#ifndef NOTHREAD
		std::atomic<bool> stop;
		static void* threadFunction(WakeUp* arg) {
			while(!arg->stop){}
			std::cout<<"by\n";
			return 0;
		}
	
		std::thread* m_thread;
#endif	
};

#endif
