#ifndef WAKEUP
#define WAKEUP


#include <atomic>
#include <thread>

#include "../mainState.h"

class Wakeup : public State
{

	public:
		Wakeup(StateData &stateData);
		~Wakeup();
		bool stillValid();
		void updateOnSensors();

	

//	private:
		//std::thread* m_thread;
//    static void* threadFunction(void* arg) {
//			std::cout<<"helloa how are you?\n";
//			return 0;
//		}

};

#endif
