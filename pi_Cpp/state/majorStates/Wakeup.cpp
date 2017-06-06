#include "Wakeup.h"

using namespace std::chrono_literals;

std::condition_variable cv;
std::mutex cv_m;

static void* threadFunction(WakeUp* currentState){				
  std::unique_lock<std::mutex> lk(cv_m);	
	int time = 0;
	int bri, ct, vol;

	WakeUp* lamps = currentState;
	Mpd* mpd = currentState->data->mpd;
	MpdState* mpdState = currentState->data->mpdState;

	//wait for 20 seconds or till the cv is notified and stop is set to true
	while(!currentState->stop.load()){
		cv.wait_for(lk, 200*100ms, [currentState](){return currentState->stop.load();});
		

		bri = (int)(BRI_PER_SEC*time);
		ct = (int)(CT_MIN+CT_PER_SEC*time);
		//Do something with lamps
		lamps->setState(lmp::BUREAU, "{\"bri\": "+bri+", \"ct\": "+ct+", \"transitiontime\": 0}")
		lamps->setState(lmp::RADIATOR, "{\"bri\": "+bri+", \"ct\": "+ct+", \"transitiontime\": 0}")

		if(bri>20)
			if(time>20/BRI_PER_SEC+5)
				lamps->setState(lmp::DOOR, "{\"bri\": "+(bri+5*BRI_PER_SEC)+", \"ct\": "+ct+", \"transitiontime\": 50}")
			else
				lamps->setState(lmp::DOOR, "{\"bri\": "+bri+", \"ct\": "+ct+", \"transitiontime\": 0}")

		if(bri>100)
			if(time>100/BRI_PER_SEC+5){
				lamps->setState(lmp::KITCHEN, "{\"bri\": "+bri+(bri+5*BRI_PER_SEC)", \"ct\": "+ct+", \"transitiontime\": 50}")
				lamps->setState(lmp::CEILING, "{\"bri\": "+bri+(bri+5*BRI_PER_SEC)", \"ct\": "+ct+", \"transitiontime\": 50}")
			}
			else{
				lamps->setState(lmp::KITCHEN, "{\"bri\": "+bri+", \"ct\": "+ct+", \"transitiontime\": 0}")
				lamps->setState(lmp::CEILING, "{\"bri\": "+bri+", \"ct\": "+ct+", \"transitiontime\": 0}")
			}

		if(time>WAKEUP_MUSIC_ON)
			if(!mpdState->playing){
				mpd->createPLFromPLs();
				mpd->sendCommandList("setvol "+VOL_MIN+" play 0");
			}
			else
				mpd->sendCommand("setvol "+(time*VOL_PER_SEC+VOL_MIN));	
	}

	std::cout<<"done with wakeup\n";
	return 0;
}

WakeUp::WakeUp(StateData* stateData)
	: State(stateData)
{
	stateName = WAKEUP_S;

	stop = false;
	m_thread = new std::thread(threadFunction, this);

	std::cout<<"Ran Wakeup state constructor\n";
	std::cout<<"stateName: "<<stateName<<"\n";
}

WakeUp::~WakeUp(){

	//send stop signal to thread
	stop = true;
	cv.notify_all();
	std::cout<<"send stop signal\n";
	m_thread->join();

	std::cout<<"cleaned up the Wakeup state\n";
}

bool WakeUp::stillValid(){
	std::cout<<"decided its still the right state\n";
	return true;
}

void WakeUp::updateOnSensors(){
	std::cout<<"updated based on sensor values and stuff\n";
}
