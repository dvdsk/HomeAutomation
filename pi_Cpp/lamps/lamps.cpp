#include "lamps.h"

Lamps::Lamps()
	: HttpGetPostPut(config::HUE_URL){

	std::string error = 
	  "[{\"error\":{\"type\":1,\"address\":\"/\",\"description\":\"unauthorized user\"}}]";
	if(get("") == error){	std::cout<<"HUE CONFIG WRONG\n";}
}

void Lamps::Off(int n){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/"+toId(n)+"/state", "{\"on\": false, \"transitiontime\": 0}");
}

void Lamps::off(){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/1/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/2/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/4/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/5/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/6/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/7/state", "{\"on\": false, \"transitiontime\": 0}");
}

void Lamps::On(int n){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/"+toId(n)+"/state", "{\"on\": true, \"transitiontime\": 0}");
}

void Lamps::on(){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/1/state", "{\"on\": true, \"transitiontime\": 0}");
	put("/lights/2/state", "{\"on\": true, \"transitiontime\": 0}");
	put("/lights/4/state", "{\"on\": true, \"transitiontime\": 0}");
	put("/lights/5/state", "{\"on\": true, \"transitiontime\": 0}");
	put("/lights/6/state", "{\"on\": true, \"transitiontime\": 0}");
	put("/lights/7/state", "{\"on\": true, \"transitiontime\": 0}");
}

void Lamps::setState(int n, std::string json){
	std::lock_guard<std::mutex> guard(lamp_mutex);

	put("/lights/"+toId(n)+"/state", json);
}

void Lamps::setState(std::string json){
	std::lock_guard<std::mutex> guard(lamp_mutex);

	put("/lights/1/state", json);
	put("/lights/2/state", json);
	put("/lights/4/state", json);
	put("/lights/5/state", json);
	put("/lights/6/state", json);
	put("/lights/7/state", json);
}


//////PRIVATE FUNCT///////////////////////

//TODO add correct numbers
inline std::string Lamps::toId(int lampNumb){
	switch(lampNumb){
		case lmp::DOOR:
			return "1";
		break;		
		case lmp::KITCHEN:
			return "2";
		break;		
		case lmp::CEILING:
			return "3";
		break;		
		case lmp::BATHROOM:
			return "4";
		break;		
		case lmp::RADIATOR:
			return "5";
		break;		
		case lmp::BUREAU:
			return "6";
		break;		
	}
	std::cout<<"ERROR -> not a known lamp\n";
	return "0";
}


