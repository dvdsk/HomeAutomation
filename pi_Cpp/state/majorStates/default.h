#ifndef DEFAULT
#define DEFAULT

#include "../mainState.h"
#include <time.h>

class Default : public State
{

	public:
		Default(StateData* stateData);
		~Default();
		bool stillValid();
		void updateOnSensors();

		std::atomic<bool> stop;
	
	private:
		std::thread* m_thread;		


};

time_t day_seconds();

#endif
