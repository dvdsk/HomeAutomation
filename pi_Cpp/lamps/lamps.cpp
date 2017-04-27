#include "lamps.h"

Lamps::Lamps()
	: HttpGetPostPut(config::HUE_URL){

	std::string error = 
	  "[{\"error\":{\"type\":1,\"address\":\"/\",\"description\":\"unauthorized user\"}}]";
	if(get("") == error){	std::cout<<"HUE CONFIG WRONG\n";}
}

void Lamps::turnOn(int lampNumb){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/1/state", "{\"on\": false}");
}



void Lamps::allon(){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/1/state", "{\"on\": true}");
	put("/lights/2/state", "{\"on\": true}");
	put("/lights/4/state", "{\"on\": true}");
	put("/lights/5/state", "{\"on\": true}");
	put("/lights/6/state", "{\"on\": true}");
	put("/lights/7/state", "{\"on\": true}");
}

void Lamps::alloff(){
	std::lock_guard<std::mutex> guard(lamp_mutex);
	
	put("/lights/1/state", "{\"on\": false}");
	put("/lights/2/state", "{\"on\": false}");
	put("/lights/4/state", "{\"on\": false}");
	put("/lights/5/state", "{\"on\": false}");
	put("/lights/6/state", "{\"on\": false}");
	put("/lights/7/state", "{\"on\": false}");
}
