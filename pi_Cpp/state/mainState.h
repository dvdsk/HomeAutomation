#ifndef MAINSTATE
#define MAINSTATE

#ifdef DEBUG
#define db(x) std::cerr << x;
#else
#define db(x)
#endif

#ifdef PRINT_STATE_MESSAGES
#define ds(x) std::cerr << x;
#else 
#define ds(x)
#endif

 
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
#include "../smallFunct/sunSetRise.h"

constexpr double LLONGITUDE = 4.497010, LLATITUDE = 52.160114;

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

enum Playback {PLAYING, PAUSED, STOPPED};
struct MpdState{
	std::atomic<Playback> playback;
	std::atomic<std::int8_t> volume;
	std::atomic<int> playlistlength;
};

struct HttpState{
	HttpState(){updated=false;}
	~HttpState(){std::cout<<"HttpState HAS BEEN DESTORYED\n";}
	std::mutex m;
	std::string url;
	std::atomic<bool> updated;
};

struct SensorState{
	SensorState(){
		lightValues_updated = false;
		tempValues_updated = false;
		humidityValues_updated = false;
		soilHumidity_updated = false;
		CO2ppm_updated = false;
		Pressure_updated = false;
		movement_updated = false;
	}

	std::atomic<int> lightValues[lght::LEN];
	std::atomic<bool> lightValues_updated; 		
	std::atomic<int> tempValues[temp::LEN];
	std::atomic<bool> tempValues_updated; 		
	std::atomic<int> humidityValues[hum::LEN];
	std::atomic<bool> humidityValues_updated; 		
	std::atomic<int> soilHumidityValues[plnt::LEN];
	std::atomic<bool> soilHumidity_updated; 		
	std::atomic<std::int32_t> movement[mov::LEN];
	std::atomic<bool> movement_updated;
	std::atomic<int> CO2ppm;
	std::atomic<bool> CO2ppm_updated;
	std::atomic<int> Pressure;
	std::atomic<bool> Pressure_updated;
};

struct SignalState{
	std::mutex m;
	std::condition_variable cv;
	bool signalled;

	void runUpdate(){
		//std::lock_guard<std::mutex> lock(m); //TODO not needed/could cause problems
		//db("\033[1;31mstarted signalling\033[0m\n")
		signalled = true;
		cv.notify_one();
		//db("\033[1;32mdone signalling\033[0m\n")
	}
};


class StateData : public Lamps
{
	public:
		StateData(SensorState* sensorState_, MpdState* mpdState_, Mpd* mpd_, 
		          HttpState* httpState_, ComputerState* computerState_, SignalState* signalState_)
		: Lamps(){
			sensorState = sensorState_;
			mpdState = mpdState_;
			mpd = mpd_;
			httpState = httpState_;
			computerState = computerState_;
			signalState = signalState_;

			//extra pointers

			double sunRise, sunSet;

			time_t theTime = time(NULL);
			struct tm *aTime = localtime(&theTime);

			sun_rise_set(aTime->tm_year+1900, aTime->tm_mon+1, aTime->tm_mday, 
			LLONGITUDE, LLATITUDE, &sunRise, &sunSet);

			tWarm = 3600*(sunSet-1); 	//time since midnight in sec UTC
			tCool = 3600*(sunRise-1);	//time since midnight in sec UTC	
		}
		~StateData(){
			std::cout<<"STATEDATA HAS BEEN DELETED\n";
		}

		SensorState* sensorState;
		MpdState* mpdState;
		ComputerState* computerState;
		Mpd* mpd; //needed to call mpd functions
		HttpState* httpState;
		SignalState* signalState;
	
		uint32_t tWarm;
		uint32_t tCool;

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
	StateData* data;

	protected:
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
