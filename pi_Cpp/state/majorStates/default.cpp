#include "default.h"

using namespace std::chrono_literals;

std::condition_variable cv_default;
std::mutex cv_default_m;

inline int fade(int y0, int x0, int x, int dyDx){
	return y0+(x-x0)*dyDx;
}

static void lightColor_thread(Default* currentState){
  std::unique_lock<std::mutex> lk(cv_default_m);	
	uint32_t time;

	uint32_t* tWarm = &currentState->data->tWarm;
	uint32_t* tCool = &currentState->data->tCool;


	constexpr int FADE_TO_EVENING = 1*3600; //seconds
	constexpr int CT_EVENING = CT_MAX-100;
	constexpr int BRI_EVENING = BRI_MAX;

	constexpr int FADE_TO_NIGHT = 1*3600;		//seconds
	constexpr int CT_NIGHT = CT_MAX;
	constexpr int BRI_NIGHT = 150;

	constexpr int FADE_TO_DAY = 1*3600;
	constexpr int CT_DAY = CT_MIN+ 110;
	constexpr int BRI_DAY = BRI_MAX;

	while(!currentState->stop.load()){

		time = day_seconds();
		//time = (uint32_t)(20.2*3600); DEBUG

		int bri;
		int ct;

		if((time > *tWarm) && (time < *tWarm+FADE_TO_EVENING)){ 			//eveningFade
			//fade from CT_DAY to CT_EVENING in FADE_TO_EVENING seconds
			ct = fade(CT_DAY, *tWarm, time, (CT_DAY-CT_EVENING)/FADE_TO_EVENING);
			bri = BRI_EVENING;
		}
		else if((time > *tWarm+FADE_TO_EVENING) 
		&& (time < *tWarm+FADE_TO_EVENING+FADE_TO_NIGHT)){						//nightFade
			//fade from CT_EVENING to CT_NIGHT in FADE_TO_NIGHT seconds
			ct = fade(CT_EVENING, *tWarm+FADE_TO_NIGHT, time, 
					 (CT_EVENING-CT_NIGHT)/FADE_TO_NIGHT);

			bri =fade(BRI_EVENING, *tWarm+FADE_TO_NIGHT, time, 
					 (BRI_EVENING-BRI_NIGHT)/FADE_TO_NIGHT);
		}
		else if((time > *tCool) && (time < *tCool+FADE_TO_DAY)){			//dayFade
			//fade from CT_NIGHT to CT_DAY in FADE_TO_NIGHT seconds
			ct = fade(CT_NIGHT, *tCool, time, 
					 (CT_NIGHT-CT_DAY)/FADE_TO_DAY);

			bri =fade(BRI_NIGHT, *tCool, time, 
					 (BRI_NIGHT-BRI_DAY)/FADE_TO_DAY);
		}
		else if((time >= *tCool+FADE_TO_DAY) && (time <= *tWarm)){		//day
			//set day value
			ct = CT_DAY;
			bri = BRI_DAY;
		}
		else{																													//night
			//set night value
			ct = CT_NIGHT;
			bri = BRI_NIGHT;
		}

		currentState->data->set_ctBri(ct, bri);	
		cv_default.wait_for(lk, 60*1s, [currentState](){return currentState->stop.load();});
	}
}				

Default::Default(StateData* stateData)
	: State(stateData){
	stateName = DEFAULT_S;	

	stop = false;
	m_thread = new std::thread(lightColor_thread, this);

	
	std::cout<<"Ran default state constructor"<<"\n";
}

Default::~Default(){
	stop = true;
	cv_default.notify_all();
	m_thread->join();
	delete m_thread;

	std::cout<<"cleaned up the default state"<<"\n";
}

bool Default::stillValid(){
	//std::cout<<"decided its still the right state"<<"\n";
	return true;
}

void Default::updateOnSensors(){
	//std::cout<<"updated based on sensor values and stuff"<<"\n";
}

time_t day_seconds(){
  time_t t1, t2;
  struct tm tms;
  time(&t1);
  gmtime_r(&t1, &tms);
  tms.tm_hour = 0;
  tms.tm_min = 0;
  tms.tm_sec = 0;
  t2 = mktime(&tms);
  return uint32_t(t1 - t2);
}

