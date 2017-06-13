#include "Wakeup.h"
#include <ctime> //debug!

using namespace std::chrono_literals;

std::condition_variable cv_wakeup;
std::mutex cv_wakeup_m;

static void* threadFunction(WakeUp* currentState){				
  std::unique_lock<std::mutex> lk(cv_wakeup_m);	
	int time = 0;
	int bri, ct;

	StateData* lamps = currentState->data;
	Mpd* mpd = currentState->data->mpd;
	MpdState* mpdState = currentState->data->mpdState;

	constexpr int DOORLAMPON = 			(int)(WAKEUP_DURATION/5); 	//sec
	constexpr int ALLLAMPSON = 			(int)(WAKEUP_DURATION/3);		//sec
	constexpr int WAKEUP_MUSIC_ON = (int)(WAKEUP_DURATION/2);		//sec

	//turn all lamps on with zero brightness
	lamps->setState(lmp::BUREAU,"{\"on\": true, \"bri\": 0, \"transitiontime\":0}");
	lamps->setState(lmp::RADIATOR,"{\"on\": true, \"bri\": 0, \"transitiontime\":0}");

	while(!currentState->stop.load() && time < WAKEUP_DURATION){
		bri = (int)(BRI_PER_SEC*time);
		ct = (int)(CT_MIN+CT_PER_SEC*time);
		//Do something with lamps
		lamps->setState(lmp::BUREAU, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
		lamps->setState(lmp::RADIATOR, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");

		if(bri>DOORLAMPON){
			if(time>DOORLAMPON/BRI_PER_SEC+10){
				std::cout<<"door lamp, updating brict\n";
				lamps->setState(lmp::DOOR, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");}
			else{
				std::cout<<"door lamp, turning on\n";
				lamps->setState(lmp::DOOR, "{\"on\": true, \"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");}
		}

		if(bri>ALLLAMPSON){
			if(time>ALLLAMPSON/BRI_PER_SEC+10){
				std::cout<<"kitchCeil, updating brict\n";
				lamps->setState(lmp::KITCHEN, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
				lamps->setState(lmp::CEILING, "{\"bri\": "+std::to_string(bri)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 0}");
			}
			else{
				std::cout<<"kitchCeil, turning on\n";
				lamps->setState(lmp::KITCHEN, "{\"on\": true, \"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");
				lamps->setState(lmp::CEILING, "{\"on\": true, \"bri\": "+std::to_string(bri+5*BRI_PER_SEC)+", \"ct\": "+std::to_string(ct)+", \"transitiontime\": 50}");
			}
		}
		if(time>WAKEUP_MUSIC_ON){
			if(mpdState->playback != PLAYING){
				std::string cList = "setvol "+std::to_string(VOL_MIN)+"\nplay 0\n";
				mpd->sendCommandList(cList);
			}
			else
				mpd->sendCommand("setvol "+std::to_string(time*VOL_PER_SEC+VOL_MIN));	
		}
	time +=5; //due to code execution +- 1 second drift over 15 min 
	std::cout<<"time: "<<time<<"\n";
	cv_wakeup.wait_for(lk, 5*1s, [currentState](){return currentState->stop.load();});
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
	stateData->mpd->saveAndClearCP();	
	stateData->mpd->QueueFromPLs("calm", 3*60, 5*60);
	stateData->mpd->QueueFromPLs("energetic", 10*60, 11*60);

	std::cout<<"Ran Wakeup state constructor\n";
	std::cout<<"stateName: "<<stateName<<"\n";
}

WakeUp::~WakeUp(){

	//send stop signal to thread
	stop = true;
	cv_wakeup.notify_all();
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
