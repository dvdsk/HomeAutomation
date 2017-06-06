#ifndef WAKEUP_G
#define WAKEUP_G

#include "../mainState.h"
#include <chrono>
#include "../../mpd/mpd.h"


class WakeUp : public State
{

	public:
		WakeUp(StateData* stateData);
		~WakeUp();
		bool stillValid();
		void updateOnSensors();

		std::atomic<bool> stop;
	
	private:

//		static void* threadFunction(WakeUp* arg);

	
		std::thread* m_thread;
};

#endif
