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
#include <stdlib.h> //syscall to 'at'

#include "../config.h"
#include "../lamps/lamps.h"

class Mpd;
//#include "../mpd/mpd.h"

//here stop is a special state  that triggers shutdown for the thread
//watchforupdates function.
enum MajorStates {
	AWAY, 					//env_alarm+plnts_alarm+intruder_alarm
	DEFAULT_S,			//env_alarm+plnts_alarm+lamps_cb+lampcheck(Kitchen, Door, Bureau, Bathroom) 
	GOINGTOSLEEP_S, 	//env_alarm+plnts_alarm
	SLEEPINTERRUPT_S,	//env_alarm+plnts_alarm
	SLEEPING, 				//env_alarm+plnts_alarm+night_intruder_alarm
	MINIMAL_S,			  //env_alarm+plnts_alarm+lampcheck(Bathroom)
	WAKEUP_S					//env_alarm+plnts_alarm+lampcheck(Bathroom)
};

struct MinorStates{
	std::atomic<bool> alarmDisarm;
	std::atomic<bool> authorisedClose;
	std::atomic<bool> listenToAudioBook;
	std::atomic<bool> wakingUp;
  std::atomic<bool> inBathroom;
  std::atomic<bool> showering;
  std::atomic<bool> inKitchenArea;
  std::atomic<bool> movieMode;
};

struct ComputerState{
	std::atomic<bool> windows;
	std::atomic<bool> linux;
	std::atomic<bool> off;
};

struct MpdState{
	std::atomic<bool> playing;
	std::atomic<bool> paused;
	std::atomic<bool> stopped;
	std::atomic<std::int8_t> volume;
};

struct HttpState{
	HttpState(){updated=false;}
	~HttpState(){std::cout<<"HttpState HAS BEEN DESTORYED\n";}
	std::mutex m;
	std::string url;
	//FIXME std::atomic<bool> updated;
	bool updated;
};

struct SensorState{
	std::atomic<int> lightValues[lght::LEN];
	std::atomic<bool> lightValues_updated; 		
	std::atomic<int> tempValues[temp::LEN];
	std::atomic<bool> tempValues_updated; 		
	std::atomic<int> humidityValues[hum::LEN];
	std::atomic<bool> humidityValues_updated; 		
	std::atomic<int> soilHumidityValues[plnt::LEN];
	std::atomic<bool> soilHumidity_updated; 		
	std::atomic<std::int32_t> movement[mov::LEN];
	std::atomic<int> CO2ppm;
	std::atomic<bool> CO2ppm_updated;
	std::atomic<int> Pressure;
	std::atomic<bool> Pressure_updated;
};

struct SignalState{
	std::mutex m;
	std::condition_variable cv;

	void runUpdate(){
		std::cout<<"done\n";
		std::unique_lock<std::mutex> lk(m);
		cv.notify_one();
	}
};


class StateData : public Lamps
{
	public:
		StateData(SensorState* sensorState_, MpdState* mpdState_, Mpd* mpd_, 
		          HttpState* httpState_, ComputerState* computerState_)
		: Lamps(){
			sensorState = sensorState_;
			mpdState = mpdState_;
			mpd = mpd_;
			httpState = httpState_;
			computerState = computerState_;
			testInt = 42;
		}
		~StateData(){
			std::cout<<"STATEDATA HAS BEEN DELETED\n";
		}

		SensorState* sensorState;
		MpdState* mpdState;
		ComputerState* computerState;
		Mpd* mpd; //needed to call mpd functions
		HttpState* httpState;

		int testInt;
		uint32_t currentTime;
		uint32_t timeStateStarted;
		
		//stateBookKeeping
		std::atomic<MajorStates> newState;		
		std::atomic<MajorStates> lastState;
		MinorStates minorState;
};

class State
{	
	public:
	State(StateData* stateData_){
		data = stateData_;
	}
	
	virtual bool stillValid() =0;
	virtual void updateOnSensors() =0;
	virtual ~State() = default;
	bool updateOnHttp();

	std::atomic<MajorStates> stateName;//FIXME (atomic needed??)

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
inline void setAlarm(int nMinutes);
inline void sleep(int seconds);
inline std::string toTime(uint32_t seconds);
inline bool setting_up_values_done();

#endif // MAINSTATE
