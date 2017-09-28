#ifndef LAMPS
#define LAMPS

#include <iostream> //cout
#include <string.h> //strcmp
#include <atomic>
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
	void on(uint8_t n);
	void on();
	/* turn off specific lamp or all lamps with zero transition time*/
	void off(uint8_t n);
	void off();

	/* set full config for one or all lamps, the configuration is not stored
		 in this class*/
	void setState(uint8_t n, std::string json);
	void setState(std::string json);

	/* set properties for lamps */
	void set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_);
	void set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime);
	void set_ctBri_f(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on);
	/* also check if the properties were set correctly */
	void set_ctBri(uint8_t n, uint8_t bri_, uint16_t ct_){
		set_ctBri_f(n, bri_, ct_);
		checkState(n);
	}
	void set_ctBri(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime){
		set_ctBri_f(n, bri_, ct_, transitionTime);
		checkState(n);
	}
	void set_ctBri(uint8_t n, uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on){
		set_ctBri_f(n, bri_, ct_, transitionTime, on);
		checkState(n);
	}
	
	void setAll_ctBri_f(uint8_t bri_, uint16_t ct_);
	void setAll_ctBri_f(uint8_t bri_, uint16_t ct_, uint8_t transitionTime);
	void setAll_ctBri_f(uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on);
	/* also check if the properties were set correctly */
	void setAll_ctBri(uint8_t bri_, uint16_t ct_){
		setAll_ctBri_f(bri_, ct_);
		checkState();
	}
	void setAll_ctBri(uint8_t bri_, uint16_t ct_, uint8_t transitionTime){
		setAll_ctBri_f(bri_, ct_, transitionTime);
		checkState();
	}
	void setAll_ctBri(uint8_t bri_, uint16_t ct_, uint8_t transitionTime, bool on){
		setAll_ctBri_f(bri_, ct_, transitionTime, on);
		checkState();
	}

	/* returns if most lights are on */
	bool avgOn();
	bool isOn[lmp::LEN];

	private:
	/* need a mutex as we may never share the same handle in multiple threads */
	std::mutex lamp_mutex;

	/* save bri, ct, xy and colormode */
	void saveState(uint8_t n);
	void saveState();

	/* special version of saveState that also checks the on/off state */
	void saveFullState(uint8_t n);
	void saveFullState();

	void checkState(uint8_t n);
	void checkState();

	std::string colormode[lmp::LEN];
	uint16_t ct[lmp::LEN];
	uint8_t bri[lmp::LEN];
	float x[lmp::LEN];
	float y[lmp::LEN];

	/* translates between lampNumb and lampId */
	std::string toId(uint8_t lampNumb);
	int toIntId(uint8_t lampNumb);
};


#endif // LAMPS
