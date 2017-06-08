#include "Wakeup.h"

using namespace std::chrono_literals;

std::condition_variable cv;
std::mutex cv_m;

static void* threadFunction(WakeUp* currentState){				
  std::unique_lock<std::mutex> lk(cv_m);	
	int time = 0;
	int bri, ct;

	StateData* lamps = currentState->data;
	Mpd* mpd = currentState->data->mpd;
	MpdState* mpdState = currentState->data->mpdState;

	while(!currentState->stop.load()){
		cv.wait_for(lk, 5*1s, [currentState](){return currentState->stop.load();});

		bri = (int)(BRI_PER_SEC*time);
		ct = (int)(CT_MIN+CT_PER_SEC*time);
		//Do something with lamps
		lamps->setState(lmp::BUREAU, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
		lamps->setState(lmp::RADIATOR, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");

		if(bri>20)
			if(time>20/BRI_PER_SEC+5)
				lamps->setState(lmp::DOOR, "{\"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");
			else
				lamps->setState(lmp::DOOR, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");

		if(bri>100)
			if(time>100/BRI_PER_SEC+5){
				lamps->setState(lmp::KITCHEN, "{\"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");
				lamps->setState(lmp::CEILING, "{\"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");
			}
			else{
				lamps->setState(lmp::KITCHEN, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
				lamps->setState(lmp::CEILING, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
			}

		if(time>WAKEUP_MUSIC_ON)
			if(mpdState->playback == PLAYING){
				std::string cList = "setvol "+std::to_string(VOL_MIN)+"\nplay 0";
				mpd->sendCommandList(cList);
			}
			else
				mpd->sendCommand("setvol "+std::to_string(time*VOL_PER_SEC+VOL_MIN));	
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

	//TODO save current playlist, clear playlist
	mpd->QueueFromPLs("calm", 3*60, 5*60);
	mpd->QueueFromPLs("energetic", 10*60, 11*60);

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
