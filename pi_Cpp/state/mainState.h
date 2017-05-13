#ifndef MAINSTATE
#define MAINSTATE

#include <ctime> //time()
#include <thread>

#include <mutex>
#include <condition_variable>
#include <memory> //for shared_ptr
#include <atomic>
#include <array>
#include <string.h> //strcmp
#include <iostream> //cout

#include "../config.h"
#include "../lamps/lamps.h"

class Mpd;
//#include "../mpd/mpd.h"

//here stop is a special state  that triggers shutdown for the thread
//watchforupdates function.
enum MajorStates {
	AWAY, 					//env_alarm+plnts_alarm+intruder_alarm
	DEFAULT,				//env_alarm+plnts_alarm+lamps_cb+lampcheck(Kitchen, Door, Bureau, Bathroom) 
	ALMOSTSLEEPING, //env_alarm+plnts_alarm
	SLEEPING, 			//env_alarm+plnts_alarm+night_intruder_alarm
	MINIMAL,			  //env_alarm+plnts_alarm+lampcheck(Bathroom)
	WAKEUP					//env_alarm+plnts_alarm+lampcheck(Bathroom)
};

struct MinorStates{
	bool alarmDisarm;
	bool authorisedClose;
	bool listenToAudioBook;
	bool wakingUp;
  bool inBathroom;
  bool showering;
  bool inKitchenArea;
  bool movieMode;
};

struct MpdState{
	std::mutex m;

	bool playing;
	bool paused;
	bool stopped;
	uint8_t volume;
};

struct HttpState{
	std::mutex m;
	std::string url;
	bool updated;
};

struct SensorState{
	std::mutex m;

	int lightValues[lght::LEN];
	bool lightValues_updated; 		
	int tempValues[temp::LEN];
	bool tempValues_updated; 		
	int humidityValues[hum::LEN];
	bool humidityValues_updated; 		
	int soilHumidityValues[plnt::LEN];
	bool soilHumidity_updated; 		
	uint32_t movement[mov::LEN];
	int CO2ppm;
	bool CO2ppm_updated;
	int Pressure;
	bool Pressure_updated;
};

struct SignalState{
	std::mutex m;
	std::condition_variable cv;

	void runUpdate(){
		std::unique_lock<std::mutex> lk(m);
		cv.notify_one();
	}
};


class StateData : Lamps
{
	public:
		StateData(SensorState* sensorState_, MpdState* mpdState_, Mpd* mpd_)
		: Lamps(){
			sensorState = sensorState_;
			mpdState = mpdState_;
			mpd = mpd_;
		}

		SensorState* sensorState;
		MpdState* mpdState;
		Mpd* mpd; //needed to call mpd functions

		uint32_t currentTime;
		uint32_t timeStateStarted;
		
		//stateBookKeeping
		MajorStates newState;		
		MajorStates lastState;
		MinorStates minorState;
};

class State
{	
	public:
		State(StateData* stateData_){
			data = stateData_;
		}
	
	virtual bool stillValid();
	virtual void updateOnSensors();
	virtual ~State();

	protected:
		StateData* data;

		//away functions in away.cpp
		void away_intruder_alarm();
		void check_Plants();

		//default functions in default.cpp
		void def_lampcheck_Door();
		void def_lampCheck_Kitchen();
		void def_lampCheck_CeilingAndRadiator();
		void def_lampCheck_Bureau();
		void lampCheck_Bathroom(); //used in other major states too
		void environmental_alarm();
		 
		//general support functions that need access to this class
		inline bool recent(uint32_t time, unsigned int threshold);
		inline bool anyRecent(uint32_t times[], unsigned int threshold);
};
	
//general support functions
inline void sleep(int seconds);
inline std::string toTime(uint32_t seconds);
inline bool setting_up_values_done();

#endif // MAINSTATE
