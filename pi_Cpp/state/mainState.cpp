#include "mainState.h"


////////////////////////GENERAL FUNCT///////////////////////////////////////
inline void sleep(int seconds){
	std::this_thread::sleep_for(std::chrono::seconds(seconds));
}

inline bool State::recent(uint32_t time, unsigned int threshold){
	if(data->currentTime-time > threshold){return true; }
	else{return false; }
}

inline bool State::anyRecent(uint32_t times[],
unsigned int threshold){
	bool recent = false;
	for(int i=0; i<mainState::LEN_movement; i++)
		if(data->currentTime - times[i] < threshold){recent = true; } 
	return recent;
}

inline std::string toTime(uint32_t seconds){
	if(seconds<3600){return std::to_string(seconds/60)+"minutes"; }
	else if(seconds<24*3600){return std::to_string(seconds/3600)+"hours"; }
	else{return std::to_string(seconds/(24*3600))+"days"; }
} 
