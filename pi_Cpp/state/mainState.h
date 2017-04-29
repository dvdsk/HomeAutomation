#ifndef MAINSTATE
#define MAINSTATE

	/* An automatic update will be triggerd whenever there is new data
	 * (from sensors). It then can send out a command. Commands can cause
	 * a change of state that can trigger an automatic update issueing
	 * a command again.
	 * 
	 * To minimise the needed computations new data is always combined 
	 * with an integer that by means of bitwise operations (1 for true 
	 * 0 for false) indicates which values have changed. This to 
	 * facilitate switching in a case statement.
	 * 
	 * Data is also not always updated, in case of a slight variation 
	 * to the origional data updating is ignored. The definition of a
	 * slight variation is left to the function providing the data.
	 * 
	 * Automatic updates consist of 2 phases, a pre scan phase that
	 * determins which states could be affected and an update function
	 * that updates the possibly affected states update function.
	 * 
	 * Example: there is a change in the brightness value for one of the
	 * lamps. If the change is large enough the function reading the value
	 * will wake up the pre scan thread of this class.
	 *
	 * Data races are prevented by the functions of this class, the class
	 * can safely be copied
	 */
	 
	 
	/* States are devided into two groups, the mutually exculusive 
	 * major states and the non exclusive minor states. A major state
	 * might be sleeping or default. In the sleeping state there should 
	 * be a diffrent response to many changing values then in the default
	 * state. The minor states can refine responses, think off showering
	 * going to the toilet etc.
	 * 
	 */

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
class Mpd;
//#include "../mpd/mpd.h"

//here stop is a special state  that triggers shutdown for the thread
//watchforupdates function.
enum MajorStates {AWAY, DEFAULT, ALMOSTSLEEPING, SLEEPING, MINIMAL};

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
	bool playing;
	bool paused;
	bool stopped;
	uint8_t volume;
};

class MainState;

//function that starts the class member thread_watchForUpdates()
void thread_state_manager(std::shared_ptr<MainState> state, std::shared_ptr<Mpd> mpd,
     std::shared_ptr<std::atomic<bool>> notShuttingdown);
	 
class MainState{
		
	public:
		//creates shared objects
		MainState();
			
		//gets data in the form of url's transformes it to commands or
		//state changes and if the state changed executes an update ran
		//in the httpd thread
		void httpSwitcher(const char* raw_url);
		
		//send commands to the right threads/functions
		void parseCommand(Command toParse);
		
		//is waken and then executes pre_scan();
		void watchForUpdate(std::shared_ptr<Mpd> mpd, 
		     std::shared_ptr<std::atomic<bool>> notShuttingdown);
		//signals the above function to run an update
		void signalUpdate();

		//sensorValues (needs to be accessible from decode)	
		std::mutex sensorVal_mutex;
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

		std::mutex mpdState_mutex;
		MpdState mpdState;

		std::array<bool, lmp::LEN> lampOn;
	
	private:
		std::mutex alarmProcedureStarted;
		uint32_t currentTime;
		uint32_t lastBedMovement;
		
		//update thread wakeup and stop mechanism
		bool stop;
		bool is_ready;
		std::mutex m;
		std::condition_variable cv;
		
		//stateBookKeeping
		MinorStates minorState;
		MajorStates majorState;		
		MajorStates lastState;
		uint32_t timeMinimalStarted;
		
		//4 mutually exclusive paths for checking which conditions should
		//be checked by the updating functions
		void update_away();			
		void update_sleeping();
		void update_default();
		void update_almostSleeping();
		void update_minimal();
		
		//functions that should be ran when changed into this state
		void init_away();
		void init_sleeping();
		void init_default();
		void init_almostSleeping(MajorStates fromState);
		void init_minimal(MajorStates fromState);
		
		//state mutually exclusive state transition functions, they are
		//ran on every check.
		void transitions_away();
		void transitions_sleeping();
		void transitions_default();
		void transitions_almostSleeping();
		void transitions_minimal();

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

		//almostSleeping functions in almostSleeping.cpp
		void almostSleeping_lampCheck();
		
		//sleeping functions in sleeping.cpp
		void night_alarm();
		
		
		//functions that change states
		 //turn lamps on
		 
		 //turn lamps off
		 
		 //movie mode
		 
		//general support functions that need access to this class
		inline bool recent(uint32_t time, unsigned int threshold);
		inline bool anyRecent(uint32_t times[], unsigned int threshold);
};
	
//general support functions
inline void sleep(int seconds);
inline std::string toTime(uint32_t seconds);
inline bool setting_up_values_done();

#endif // MAINSTATE
