#include <iostream> //cout
#include "lamps.h"

int main(){
	Lamps lamps;
	lamps.setState(lmp::KITCHEN, "{\"on\": true, \"bri\": 200, \"transitiontime\": 0}");
}
