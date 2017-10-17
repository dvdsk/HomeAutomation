#include "lamps.h"

Lamps::Lamps()
	: HttpSocket(config::HUE_IP, 80){

	std::string error =
	  "[{\"error\":{\"type\":1,\"address\":\"/\",\"description\":\"unauthorized user\"}}]";

	std::string test;
	if(get(BASE_URL) == error){	std::cout<<"HUE CONFIG WRONG\n";}

	//get current settings for all lamps and store
	saveFullState();

}

void Lamps::off(uint8_t n){
	std::string resource;
	std::lock_guard<std::mutex> guard(lamp_mutex);
	saveState(n);

	isOn[n] = false;
	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)
		       +(std::string)+"/state";
	put(resource, "{\"on\": false, \"transitiontime\": 0}");

	startCheck(n);
	checkON(n);
	finishCheck(n);
}

void Lamps::off(){
	std::string resource;
	std::lock_guard<std::mutex> guard(lamp_mutex);
	saveState();

	for(int n=0; n<lmp::LEN; n++){
		isOn[n] = false;
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)
						 +(std::string)+"/state";
		put(resource, "{\"on\": false, \"transitiontime\": 0}");

		startCheck(n);
		checkON(n);
		finishCheck(n);
	}
}

inline void Lamps::saveState(uint8_t n){
	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n);
	std::string state = get(resource);

	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	bri[n] = stoi(state.substr(pos1+5, pos2-pos1));

	pos1 = state.find("xy", 54)+sizeof("xy\":[");
	x[n] = stof(state.substr(pos1, 5));
	y[n] = stof(state.substr(state.find(",", pos1)+sizeof(","), 5));

	pos1 = state.find("ct", 73)+sizeof("ct\"");
	ct[n] = stoi(state.substr(pos1, 3));

	pos1 = state.find("colormode", 103)+sizeof("colormode\":");
	pos2 = state.find(",", pos1)-1;
	colormode[n] = state.substr(pos1, pos2-pos1);
}

inline void Lamps::saveFullState(uint8_t n){
	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n);
	std::string state = get(resource);

	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	std::lock_guard<std::mutex> guard(lamp_mutex);
	bri[n] = stoi(state.substr(pos1+5, pos2-pos1));

	isOn[n] = (state.substr(sizeof("\"state\":{\"on\""), 4) == "true");

	pos1 = state.find("xy", 54)+sizeof("xy\":[");
	x[n] = stof(state.substr(pos1, 5));
	y[n] = stof(state.substr(state.find(",", pos1)+sizeof(","), 5));

	pos1 = state.find("ct", 73)+sizeof("ct\"");
	ct[n] = stoi(state.substr(pos1, 3));

	pos1 = state.find("colormode", 103)+sizeof("colormode\":");
	pos2 = state.find(",", pos1)-1;
	colormode[n] = state.substr(pos1, pos2-pos1);
}

void Lamps::saveState(){
	for(int n=0; n<lmp::LEN; n++)
		saveState(n);
}

void Lamps::saveFullState(){
	for(int n=0; n<lmp::LEN; n++)
		saveFullState(n);
}

void Lamps::startCheck(uint8_t n){
	std::string colormode_;
	toput = "{";

	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n);
	state = get(resource);
}

void Lamps::checkBri(uint8_t n){
	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	uint8_t bri_ = stoi(state.substr(pos1+5, pos2-pos1));

	if(bri_ != bri[n]){
		toput += "\"bri\":"+std::to_string(bri[n])+",";
		std::cout<<"bri: "<<bri_<<"\n";
	}
}

void Lamps::checkON(uint8_t n){
	bool isOn_ = (state.substr(sizeof("\"state\":{\"on\""), 4) == "true");

	if(isOn_ != isOn[n]){
		if(isOn[n])
			toput += "\"on\": true,";
		else
			toput += "\"on\": false,";
	}
}

void Lamps::checkCt(uint8_t n){
	int pos1 = state.find("ct", 73)+sizeof("ct\"");
	uint16_t ct_ = stoi(state.substr(pos1, 3));

	if(ct_ != ct[n]){
		toput += "\"ct\":"+std::to_string(ct[n])+",";
	}
}

void Lamps::checkColor(uint8_t n){
	int pos1 = state.find("colormode", 103)+sizeof("colormode\":");
	int pos2 = state.find(",", pos1)-1;
	std::string colormode_ = state.substr(pos1, pos2-pos1);

	if(colormode_.compare(colormode[n]) != 0){
		if(colormode_.compare("ct") == 0)
			toput += "\"ct\":"+std::to_string(ct[n])+",";
		if(colormode_.compare("xy") == 0)
			toput += "\"xy\":["+std::to_string(x[n])+","+std::to_string(y[n])+"],";
		else
			std::cerr<<"colormodes other then ct and xy not implemented";
	}//check if colormode set correct
	else{
		if(colormode[n].compare("ct") == 0){
			pos1 = state.find("ct", 73)+sizeof("ct\"");
			uint16_t ct_ = stoi(state.substr(pos1, 3));
			if(ct_ != ct[n]){
				toput += "\"ct\":"+std::to_string(ct[n])+",";
			}
		}//check ct
		else if(colormode[n].compare("xy") == 0){
			pos1 = state.find("xy", 54)+sizeof("xy\":[");
			float x_ = stof(state.substr(pos1, 5));
			float y_ = stof(state.substr(state.find(",", pos1)+sizeof(","), 5));
			if(x_ != x[n] || y_ != y[n]){
				toput += "\"xy\":["+std::to_string(x[n])+","+std::to_string(y[n])+"],";
			}
		}//check xy
	}//check xy or ct?
}

void Lamps::finishCheck(uint8_t n){
	if(toput.length()>2){
		toput.pop_back();
		toput += "}";
		std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, toput);
	}
}

void Lamps::checkState(uint8_t n){
	float x_, y_;
	uint16_t ct_;
	uint8_t bri_;
	bool isOn_;
	std::string colormode_, toput = "{";

	std::string resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n);
	std::string state = get(resource);

	int pos1 = state.find("bri");
	int pos2 = state.find(",",pos1);
	bri_ = stoi(state.substr(pos1+5, pos2-pos1));
	isOn_ = (state.substr(sizeof("\"state\":{\"on\""), 4) == "true");

	pos1 = state.find("xy", 54)+sizeof("xy\":[");
	x_ = stof(state.substr(pos1, 5));
	y_ = stof(state.substr(state.find(",", pos1)+sizeof(","), 5));

	pos1 = state.find("ct", 73)+sizeof("ct\"");
	ct_ = stoi(state.substr(pos1, 3));

	pos1 = state.find("colormode", 103)+sizeof("colormode\":");
	pos2 = state.find(",", pos1)-1;
	colormode_ = state.substr(pos1, pos2-pos1);

	std::lock_guard<std::mutex> guard(lamp_mutex);
	if(bri_ != bri[n])
		toput += "\"bri\":"+std::to_string(bri[n])+",";
	if(isOn_ != isOn[n]){
		if(isOn[n])
			toput += "\"on\": true,";
		else
			toput += "\"on\": false,";
	}
	if(colormode_.compare(colormode[n]) != 0)
		toput += "\"colormode\":"+colormode[n]+",";

	if(colormode_.compare("ct") == 0){
		if(ct_ != ct[n]){
			toput += "\"ct\":"+std::to_string(ct[n])+",";
		}
	}
	else{
		if(x_ != x[n] || y_ != y[n]){
			toput += "\"xy\":["+std::to_string(x[n])+","+std::to_string(y[n])+"],";
		}
	}

	if(toput.length()>2){
		toput.pop_back();
		toput += "}";
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, toput);
	}
}

void Lamps::checkState(){
	for(int n=0; n<lmp::LEN; n++)
		checkState(n);
}

void Lamps::on(uint8_t n){
	std::string resource;
	std::string toput;

	std::lock_guard<std::mutex> guard(lamp_mutex);

	isOn[n] = true;
	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
	toput =
	"{\"on\": true, \"transitiontime\": 0,\"bri\":"	+ std::to_string(bri[n])
	+ ",\"xy\":["+std::to_string(x[n])+","+std::to_string(y[n])+"]}";

	put(resource, toput);

	startCheck(n);
	checkON(n);
	checkBri(n);
	checkColor(n);
	finishCheck(n);
}

void Lamps::on(){
	std::string resource;
	std::string toput;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int n=0; n<lmp::LEN; n++){
		isOn[n] = true;
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		toput =
		"{\"on\": true, \"transitiontime\": 0,\"bri\":"	+ std::to_string(bri[n])
		+ ",\"xy\":["+std::to_string(x[n])+","+std::to_string(y[n])+"]}";

		put(resource, toput);
	}

	for(int n=0; n<lmp::LEN; n++){
		startCheck(n);
		checkON(n);
		checkBri(n);
		checkColor(n);
		finishCheck(n);
	}
}

void Lamps::setState(uint8_t n, std::string json){
	std::string resource;

	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
	put(resource, json);
}

void Lamps::setState(std::string json){
	std::string resource;

	for(int n=0; n<lmp::LEN; n++){
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, json);
	}
}

void Lamps::set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_){
	if(isOn[n]){
		std::string resource;
		std::string json;
		std::lock_guard<std::mutex> guard(lamp_mutex);

		ct[n] = ct_;
		bri[n] = bri_;
		colormode[n] = "ct";

		json = "{\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)+"}";
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, json);
	}
}

void Lamps::set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime){
	if(isOn[n]){
		std::string resource;
		std::string json;
		std::lock_guard<std::mutex> guard(lamp_mutex);

		ct[n] = ct_;
		bri[n] = bri_;
		colormode[n] = "ct";

		json = "{\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
		+", \"transitiontime\": "+std::to_string(transitionTime)+"}";
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, json);
	}
}

void Lamps::set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on){
	std::string resource;
	std::string json;
	std::string onStr = "{\"on\": true, ";
	std::string offStr = "{\"on\": false, ";
	std::lock_guard<std::mutex> guard(lamp_mutex);

	ct[n] = ct_;
	bri[n] = bri_;
	colormode[n] = "ct";
	isOn[n]= on;

	if(on)
		json = onStr+"\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
		+", \"transitiontime\": "+std::to_string(transitionTime)+"}";
	else
		json = offStr+"\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
		+", \"transitiontime\": "+std::to_string(transitionTime)+"}";

	resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
	put(resource, json);

}

void Lamps::setAll_ctBri_f(uint8_t bri_, uint16_t ct_){
	std::string resource;
	std::string json;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int n=0; n<lmp::LEN; n++){
		if(isOn[n]){
			ct[n] = ct_;
			bri[n] = bri_;
			colormode[n] = "ct";

			json = "{\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)+"}";
			resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
			put(resource, json);
		}
	}
}

void Lamps::setAll_ctBri_f(uint8_t bri_, uint16_t ct_, uint8_t transitionTime){
	std::string resource;
	std::string json;
	std::lock_guard<std::mutex> guard(lamp_mutex);

	for(int n=0; n<lmp::LEN; n++){
		if(isOn[n]){
			ct[n] = ct_;
			bri[n] = bri_;
			colormode[n] = "ct";

			json = "{\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
			+", \"transitiontime\": "+std::to_string(transitionTime)+"}";
			resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
			put(resource, json);
		}
	}
}

void Lamps::setAll_ctBri_f(uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on){
	std::string resource;
	std::string json;
	std::lock_guard<std::mutex> guard(lamp_mutex);
	std::string onStr = "{\"on\": true, ";
	std::string offStr = "{\"on\": false, ";

	for(int n=0; n<lmp::LEN; n++){
		ct[n] = ct_;
		bri[n] = bri_;
		colormode[n] = "ct";
		isOn[n]= on;
		if(on)
			json = onStr+"\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
			+", \"transitiontime\": "+std::to_string(transitionTime)+"}";
		else
			json = offStr+"\"bri\": "+std::to_string(bri_)+", \"ct\": "+std::to_string(ct_)
			+", \"transitiontime\": "+std::to_string(transitionTime)+"}";
		resource = (std::string)BASE_URL+(std::string)"/lights/"+toId(n)+"/state";
		put(resource, json);
	}
}

bool Lamps::avgOn(){
	int total=0;
	for(int i=0; i<lmp::LEN; i++)
		total += isOn[i];

	return total>0.5*lmp::LEN;
}

/////////////////////////////

//TODO add correct numbers
inline std::string Lamps::toId(uint8_t lampNumb){
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
inline int Lamps::toIntId(uint8_t lampNumb){
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
