#include <iostream> //cout
#include "lamps.h"

int main(){
	Lamps* lamps = new Lamps;
	lamps->off(1);	
	//lamps.setState(lmp::KITCHEN, "{\"on\": true, \"bri\": 200, \"transitiontime\": 0}");
}
