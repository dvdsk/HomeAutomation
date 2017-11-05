#include "Wakeup.h"
#include <ctime> //debug!

using namespace std::chrono_literals;

std::condition_variable cv_wakeup;
std::mutex cv_wakeup_m;

static void* threadFunction(WakeUp* currentState){
  std::unique_lock<std::mutex> lk(cv_wakeup_m);
	int time = 0;
	int bri, ct, volume;
  bool started = false;

	StateData* lamps = currentState->data;
	Mpd* mpd = currentState->data->mpd;
	MpdState* mpdState = currentState->data->mpdState;

	constexpr int DOORLAMPON = 			(int)(WAKEUP_DURATION/5); 	//sec
	constexpr int ALLLAMPSON = 			(int)(WAKEUP_DURATION/3);		//sec
	constexpr int WAKEUP_MUSIC_ON = (int)(WAKEUP_DURATION/2);		//sec

	//turn all lamps on with zero brightness and max colortone
	lamps->set_ctBri(lmp::BUREAU, 0, CT_MAX, 0, true);
	lamps->set_ctBri(lmp::RADIATOR, 0, CT_MAX, 0, true);

	while(!currentState->stop.load() && time < WAKEUP_DURATION+1){
		bri = (int)(BRI_PER_Ks*time/1000);
		ct = (int)(CT_MAX-CT_PER_Ks*time/1000);

		//Do something with lamps
		lamps->set_ctBri(lmp::BUREAU, bri, ct, 0);
		lamps->set_ctBri(lmp::RADIATOR, bri, ct, 0);

		if(time>DOORLAMPON){
			if(time>DOORLAMPON+UPDATEPERIOD){
				std::cout<<"door lamp, updating brict\n";
				lamps->set_ctBri(lmp::DOOR, bri, ct, 0);}
			else{
				std::cout<<"door lamp, turning on\n";
				lamps->set_ctBri(lmp::DOOR, (int)(bri+10*BRI_PER_Ks/1000), ct, 100, true);}
		}

		if(time>ALLLAMPSON){
			if(time>ALLLAMPSON+UPDATEPERIOD){
				std::cout<<"kitchCeil, updating brict\n";
				lamps->set_ctBri(lmp::KITCHEN, bri, ct, 0);
				lamps->set_ctBri(lmp::CEILING, bri, ct, 0);
			}
			else{
				std::cout<<"kitchCeil, turning on\n";
				lamps->set_ctBri(lmp::KITCHEN, (int)(bri+10*BRI_PER_Ks/1000), ct, 100, true);
				lamps->set_ctBri(lmp::CEILING, (int)(bri+10*BRI_PER_Ks/1000), ct, 100, true);
			}
		}

		if(time>WAKEUP_MUSIC_ON){
			if(mpdState->playback != PLAYING){
        if(!started){
          started = true;
          std::cout<<"turning on music\n";
          std::string cList = "setvol "+std::to_string(VOL_MIN)+"\nplay 0\n";
          mpd->sendCommandList(cList);
          mpdState->playback = PLAYING;
        }
			}
			else if(started){
				volume = (int)(((time-WAKEUP_MUSIC_ON)*VOL_PER_Ks)/1000 + VOL_MIN);
				mpd->sendCommand("setvol "+std::to_string(volume)+"\n");
				std::cout<<"volPerSec: "<<VOL_PER_Ks/1000<<"\n";
				std::cout<<"setting volume to: "<<volume<<"\n";
			}
		}
  	time += UPDATEPERIOD; //due to code execution +- 1 second drift over 15 min
  	cv_wakeup.wait_for(lk, UPDATEPERIOD*1s, [currentState](){return currentState->stop.load();});
	}
	currentState->done = true;
	currentState->data->signalState->runUpdate(); //TODO FIXME
	std::cout<<"done with wakeup\n";
	return 0;
}

WakeUp::WakeUp(StateData* stateData)
	: State(stateData)
{
	data->stateName = WAKEUP_S;
	data->newState = DEFAULT_S;

	stateData->mpd->saveAndClearCP();
	std::cout<<"\033[1;34mdone saveAndClear\033[0m\n";

	stateData->mpd->QueueFromPLs("calm", 3*60, 5*60);
	std::cout<<"\033[1;34mdone queue1\033[0m\n";
	stateData->mpd->QueueFromPLs("energetic", 10*60, 11*60);
	std::cout<<"\033[1;34mdone queue2\033[0m\n";
	stateData->mpd->QueueFromPLs("active", 30*60, 60*60);
	std::cout<<"\033[1;34mdone queue3\033[0m\n";

	stop = false;
	done = false;
	m_thread = new std::thread(threadFunction, this);
	std::cout<<"Ran Wakeup state constructor\n";
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
	//std::cout<<"decided its still the right state\n";
	return !done;
}

void WakeUp::updateOnSensors(){
	//std::cout<<"updated based on sensor values and stuff\n";
	lampCheck_Bathroom();
}
