#include "lamps.h"

Lamps::Lamps()
	: HttpSocket(config::HUE_IP, 80){

	std::string error = 
	  "[{\"error\":{\"type\":1,\"address\":\"/\",\"description\":\"unauthorized user\"}}]";

	if(get(BASE_URL) == error){	std::cout<<"HUE CONFIG WRONG\n";}

	//get current settings for all lamps and store
	saveState();
}

void Lamps::off(int n){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	saveState(n);

	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)
		+(std::string)+"/state";

	put(resource, "{\"on\": false, \"transitiontime\": 0}");
}

void Lamps::off(){
	std::string resource;
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	saveState();

	for(int n=0; n<lmp::LEN; n++){
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)
							 +(std::string)+"/state";

		put(resource, "{\"on\": false, \"transitiontime\": 0}");
	}
}

inline void Lamps::saveState(int n){
	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n);

	std::string state = get(resource);
	//std::cout<<state<<"\n";

	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	lampBri[n] = stoi(state.substr(pos1+5, pos2-pos1));

	pos1 = state.find("xy", 54)+sizeof("xy\":[");
	lampX[n] = stof(state.substr(pos1, 5));
	lampY[n] = stof(state.substr(state.find(",", pos1)+sizeof(","), 5));
}

void Lamps::saveState(){

	for(int n=0; n<lmp::LEN; n++)
		saveState(n);
}

void Lamps::on(int n){
	std::string resource;
	std::string toput;

	std::lock_guard<std::mutex> guard(lamp_mutex);

	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
	toput = 
	"{\"on\": true, \"transitiontime\": 0,\"bri\":"	+ std::to_string(lampBri[n])
	+ ",\"xy\":["+std::to_string(lampX[n])+","+std::to_string(lampY[n])+"]}";

	put(resource, toput);
}

void Lamps::on(){
	std::string resource;
	std::string toput;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int n=0; n<lmp::LEN; n++){
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		toput = 
		"{\"on\": true, \"transitiontime\": 0,\"bri\":"	+ std::to_string(lampBri[n])
		+ ",\"xy\":["+std::to_string(lampX[n])+","+std::to_string(lampY[n])+"]}";

		put(resource, toput);
	}
}

void Lamps::setState(int n, std::string json){
	std::string resource;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
	put(resource, json);
}

void Lamps::setState(std::string json){
	std::string resource;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int n=0; n<lmp::LEN; n++){
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, json);
	}
}

//////PRIVATE FUNCT///////////////////////

//TODO add correct numbers
inline std::string Lamps::toId(int lampNumb){

	switch(lampNumb){
		case lmp::DOOR: //done
			return "5";	
		case lmp::KITCHEN:
			return "7";	
		case lmp::CEILING: //done
			return "4";	
		case lmp::BATHROOM: //done
			return "1";	
		case lmp::RADIATOR: //done
			return "6";
		case lmp::BUREAU: //done
			return "2";
		default:
			std::cout<<"ERROR hiero-> "<<lampNumb<<" not a known lamp\n";	
			break;		
	}
	return "0";
}

//TODO add correct numbers
inline int Lamps::toIntId(int lampNumb){
	switch(lampNumb){
		case lmp::DOOR:
			return 5;	
		case lmp::KITCHEN:
			return 7;	
		case lmp::CEILING:
			return 4;
		case lmp::BATHROOM:
			return 1;
		case lmp::RADIATOR:
			return 6;
		case lmp::BUREAU:
			return 2;
		default:
			std::cout<<"ERROR -> "<<lampNumb<<" not a known lamp\n";	
			break;				
	}

	return 0;
}

