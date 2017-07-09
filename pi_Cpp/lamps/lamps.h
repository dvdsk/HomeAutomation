#ifndef LAMPS
#define LAMPS

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include "../smallFunct/HttpSocket.h"
#include "../config.h"

constexpr const char* BASE_URL = config::HUE_RESOURCE;

/*small wrapper around HttpGetPostPut for controlling the lamps */ 
class Lamps : public HttpSocket
{
	
	public:
	/* check if user is registerd on the bridge, if not output an error.
		 get and parse the current lamp status*/
	Lamps();

	/* turn on specific lamp or all lamps with zero transition time with
	   the last off settings */
	void on(int n);
	void on();
	/* turn off specific lamp or all lamps with zero transition time*/
	void off(int n);
	void off();

	/* set full config for one or all lamps*/
	void setState(int n, std::string json);
	void setState(std::string json);

	private:
	/* need a mutex as we may never share the same handle in multiple threads */
	std::mutex lamp_mutex;

	void saveState(int n);
	void saveState();

	int lampBri[7];
	float lampX[7];
	float lampY[7];

	/* translates between lampNumb and lampId */
	std::string toId(int lampNumb);
	int toIntId(int lampNumb);
};


#endif // LAMPS
