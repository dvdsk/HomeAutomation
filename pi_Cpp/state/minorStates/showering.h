#ifndef MAINSTATE
#define MAINSTATE


#include <thread>
#include <mutex>
#include <memory> //for shared_ptr
#include <array>
#include <string.h> //strcmp
#include <iostream> //cout
#include "../config.h"

//lamps
constexpr int l_DOOR = 0; 
constexpr int l_KITCHEN = 1;
constexpr int l_CEILING = 2;
constexpr int l_BUREAU = 3;
constexpr int l_RADIATOR = 4;
constexpr int l_BATHROOM = 5;

//movement sensors
constexpr int m_KITCHEN = 1;
constexpr int m_BEDLEFT = 2;
constexpr int m_BEDRIGHT = 3;
constexpr int m_BATHROOM = 4

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF};

struct user {
  bool not_present;
  bool sleeping;
  
	bool authorised;
  bool goingToBed;
  bool awoken;
  
  bool bedMode;
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
		std::shared_ptr<bool> lightValues_updated; 
		std::shared_ptr<std::mutex> lightValues_mutex;		
		
		//array with unix time when a movement sensor
		//was last activated.
		std::shared_ptr<std::array<uint32_t, 5>> movement;
		std::shared_ptr<std::mutex> movement_mutex;		
		
		std::shared_ptr<struct user> userState;	//len = 10;
		std::shared_ptr<std::mutex> userState_mutex;

		std::shared_ptr<std::array<bool, 6>> lampOn; //needs mutex?
		
		//4 mutually exclusive paths for checking which conditions should
		//be checked by the updating functions
		void pre_scan_notPresent();		//alarm is on;				
		void pre_scan_sleeping();			//alarm is on;
		void pre_scan_inBed();
		void pre_scan_default();
		
		//update functions started by the updated paths
		void update_basic_lights(); //dep user, dep light
		
		void update_movement_lights(); //dep user, dep light, dep movement
		
		void update_music();
		
		void update_computer();
		//end of determining functions
		
		
		
		//inline functions present for more readable code
		inline void def_lampCheck_Door();								//dep. on: l
		inline void def_lampCheck_Bureau();							//dep. on: l
		inline void def_lampCheck_CeilingAndRadiator();	//dep. on: l
		
		inline void def_lampCheck_Kitchen();	//dep. on: l,m
		inline void lampCheck_Bathroom();			//dep. on: m
		
		inline void lampCheck_outOfBed; 			//dep. on: m
	};
	
	





#endif // MAINSTATE
