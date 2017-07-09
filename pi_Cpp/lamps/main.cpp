#include <iostream> //cout
#include "lamps.h"

int main(){
	Lamps* lamps = new Lamps;


//	lamps->off(5);	
//	lamps->on(5);	

while(1){
	lamps->off();	
	lamps->on();
}
	//lamps.setState(lmp::KITCHEN, "{\"on\": true, \"bri\": 200, \"transitiontime\": 0}");
}
