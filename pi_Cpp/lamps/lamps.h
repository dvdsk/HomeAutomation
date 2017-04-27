#ifndef LAMPS
#define LAMPS

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include "httpGetPostPut.h"
#include "../config.h"


/*change lamp on/off, brightness or color commands to change
  are given from state menagement only, thus no internal callback is made
	functions will confirm if the change occured and retry a number of times
	if not */ 
class Lamps : public HttpGetPostPut
{
	
	public:
	/* check if user is registerd on the bridge, if not output an error.
		 get and parse the current lamp status*/
	Lamps();

	/* turn on all lamps*/
	void allon();
	/* turn off all lamps*/
	void alloff();

	/* turn on a specific lamp*/
	void turnOn(int lampNumb);

	/* turn off a specific lamp*/
	void turnOff(int lampNumb);

	private:
	/* need a mutex as we may never share the same handle in multiple threads */
	std::mutex lamp_mutex;
	
};


#endif // LAMPS
