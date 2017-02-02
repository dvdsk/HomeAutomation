#ifndef MAINSTATE
#define MAINSTATE


#include <thread>
#include <mutex>
#include <memory> //for shared_ptr
#include <array>
#include <string.h> //strcmp
#include <iostream> //cout

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF};

struct user {
  bool present;
	bool authorised;
	
  bool sleeping;
  bool goingToBed;
  bool awoken;
  
  bool inBed;
  bool outOfBed;
	bool showering;

  bool inBathroom;
  bool inKitchenArea;
};

struct computer {
	bool playingSound;
};

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
		void thread_watchForUpdate();
	
	private:
		//using bitwise ops to indicate if a values changed 
		std::shared_ptr<std::array<int, 5>> lightValues;
		std::shared_ptr<int> lightValues_updated; 
		std::shared_ptr<std::mutex> lightValues_mutex;
		
		std::shared_ptr<struct user> userState;	
		std::shared_ptr<int> userState_updated; //using bitwise ops indicate changes
		std::shared_ptr<std::mutex> userState_mutex;
		
		//scan the data and choose which update functions should be run
		void pre_scan();
		
		//determine if and if so which command should be send to the ..
		void update_lights();
		
		void update_music();
		
		void update_computer();
		//end of determining functions
	};
	
	





#endif // MAINSTATE
