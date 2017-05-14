#include "mainState.h"

void State::updateOnHttp(){
	std::string url;
	data->httpState->updated = false;

	url = data->httpState->url;
	data->httpState->m.unlock();//unlock to indicate url has been read

	if(url == "/|lamps/evening"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setState("{\"bri\":254, \"ct\":320, \"transitiontime\":10}");
	}
	if(url == "/|lamps/night"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setState("{\"bri\":220, \"ct\":500, \"transitiontime\":10}");
	}
	if(url == "/|lamps/bedlight"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setState("{\"bri\":1, \"ct\":500, \"transitiontime\":10}");
	}
	if(url == "/|lamps/normal"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setState("{\"bri\":254, \"ct\":220, \"transitiontime\":10}");
	}
	if(url == "/|lamps/alloff"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->Lamps::off(1);
	}
	if(url == "/|lamps/allon"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->Lamps::on(1);
	}


	if(url == "/|state/away"){
		if(stateName != AWAY){data->newState = AWAY;}
	}
	if(url == "/|state/default"){
		if(stateName != DEFAULT_S){data->newState = DEFAULT_S;}
	}
	if(url == "/|state/goingToSleep"){
		if(stateName != GOINGTOSLEEP_S){data->newState = GOINGTOSLEEP_S;}
	}
	if(url == "/|state/sleeping"){
		if(stateName != SLEEPING){data->newState = SLEEPING;}
	}
	if(url == "/|state/minimal"){
		if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
	}		
}

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
