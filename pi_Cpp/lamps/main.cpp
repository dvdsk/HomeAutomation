#include <iostream> //cout
#include "lamps.h"

int main(){
	Lamps* lamps = new Lamps;

//	lamps->off();
//	lamps->on();

	lamps->off(lmp::BATHROOM);	
	lamps->on(lmp::BATHROOM);	

while(1){
	lamps->off(lmp::BATHROOM);	
	lamps->on(lmp::BATHROOM);	
}
	//lamps.setState(lmp::KITCHEN, "{\"on\": true, \"bri\": 200, \"transitiontime\": 0}");
}
