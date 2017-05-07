#include "lamps.h"

Lamps::Lamps()
	: HttpGetPostPut(config::HUE_URL){

	std::string error = 
	  "[{\"error\":{\"type\":1,\"address\":\"/\",\"description\":\"unauthorized user\"}}]";
	if(get("") == error){	std::cout<<"HUE CONFIG WRONG\n";}
}

void Lamps::off(int n){
	std::lock_guard<std::mutex> guard(lamp_mutex);

	saveState(n);

	put("/lights/"+toId(n)+"/state", "{\"on\": false, \"transitiontime\": 0}");
}

void Lamps::off(){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	//saveState();

	put("/lights/1/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/2/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/4/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/5/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/6/state", "{\"on\": false, \"transitiontime\": 0}");
	put("/lights/7/state", "{\"on\": false, \"transitiontime\": 0}");
}

//NOT SHOULD ALWAYS BE RAN IN A MUTEX-env
void Lamps::saveState(int n){

	std::string state = get("/lights/"+toId(n));
	std::cout<<state<<"\n";

	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	lampBri[n] = stoi(state.substr(pos1+5, pos2-pos1));
	pos1 = state.find("xy", 54)+5;
	lampX[n] = stof(state.substr(pos1, 5));
	lampY[n] = stof(state.substr(state.find(",", pos1)+1, 5));
}

//NOT SHOULD ALWAYS BE RAN IN A MUTEX-env
void Lamps::saveState(){

	int pos;	
	for(int i=0; i<7; i++){ 
		std::string state = get("/lights/"+std::to_string(i+1));
		pos = state.find("bri");
		lampBri[i] = stoi(state.substr(pos+5, state.find(",", pos)));

		pos = state.find("xy", 54)+5;
		lampX[i] = stof(state.substr(pos, 5));
		lampY[i] = stof(state.substr(state.find(",", pos)+1, 5));
	}
}

void Lamps::on(int n){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	n = toIntId(n);
	
	put("/lights/"+std::to_string(n)+"/state", "{\"on\": true, \"transitiontime\": 0,\"bri\":"
	    +std::to_string(lampBri[n])+"\"xy\":["+std::to_string(lampX[n])+
			","+std::to_string(lampY[n])+"]}");

	std::cout<<"done: "<<"{\"on\": true, \"transitiontime\": 0,\"bri\":"
	    +std::to_string(lampBri[n])+"\"xy\":["+std::to_string(lampX[n])+
			","+std::to_string(lampY[n])+"]}"<<"\n";
}

void Lamps::on(){
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int i=0; i<7; i++)
		put("/lights/"+std::to_string(i+1)+"/state", "{\"on\": true, \"transitiontime\": 0,\"bri\":"
		  +std::to_string(lampBri[1])+"\"xy\":["+std::to_string(lampX[1])+
			","+std::to_string(lampY[1])+"]}");

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

//TODO add correct numbers
inline int Lamps::toIntId(int lampNumb){
	switch(lampNumb){
		case lmp::DOOR:
			return 1;
		break;		
		case lmp::KITCHEN:
			return 2;
		break;		
		case lmp::CEILING:
			return 3;
		break;		
		case lmp::BATHROOM:
			return 4;
		break;		
		case lmp::RADIATOR:
			return 5;
		break;		
		case lmp::BUREAU:
			return 6;
		break;		
	}
	std::cout<<"ERROR -> not a known lamp\n";
	return 0;
}

