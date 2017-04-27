#ifndef LAMPS
#define LAMPS

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include "httpGetPostPut.h"
#include "../config.h"


/*small wrapper around HttpGetPostPut for controlling the lamps */ 
class Lamps : public HttpGetPostPut
{
	
	public:
	/* check if user is registerd on the bridge, if not output an error.
		 get and parse the current lamp status*/
	Lamps();

	/* turn on specific lamp or all lamps with zero transition time*/
	void On(int n);
	void on();
	/* turn off specific lamp or all lamps with zero transition time*/
	void Off(int n);
	void off();

	/* set full config for one or all lamps*/
	void setState(int n, std::string json);
	void setState(std::string json);

	private:
	/* need a mutex as we may never share the same handle in multiple threads */
	std::mutex lamp_mutex;

	/* translates between lampNumb and lampId */
	std::string toId(int lampNumb);
};


#endif // LAMPS
