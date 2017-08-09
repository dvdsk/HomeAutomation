#ifndef MPD
#define MPD

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include <memory> 
#include <atomic>
#include <vector>
#include <random>
#include <ctime> 
#include <csignal>

//needed for sockets
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h> 

#include "../state/mainState.h"


constexpr int portno = 6600;
constexpr const char* hostname = "192.168.1.10";

class Mpd{
	public:
		Mpd(MpdState* mpdState_, SignalState* signalState_); //connects to mpd
		~Mpd();

		void readLoop();

		void sendCommand(std::string const &command);
		void sendCommandList(std::string &command);

		/* adds songs from the source playlist to the current playlist / queue 
		   a minimum and maximum runtime can be passed along */ 
		void QueueFromPLs(std::string const &source,
		const unsigned int tMin, const unsigned int tMax);

		/* stores the current playlist to "oldPL" then clears the current playlist*/
		void saveAndClearCP();

	private:

		std::mutex debug_mutex;
		void debugPrint(std::string toprint){
			std::lock_guard<std::mutex> guard(debug_mutex);
			std::cout<<toprint;
		}
		void safeWrite(int sockfd, const char* message, int len);		
		void processMessage(std::string output);

		MpdState* mpdState;
		SignalState* signalState;

		int sockfd;//sockfd file discriptor
    struct sockaddr_in serv_addr;
    struct hostent *server;
		std::mutex mpd_mutex;

		//needed for data requst
		std::mutex cv_m;
		std::condition_variable cv;
		bool dataRdy;
		std::atomic<bool> dataReqested;
		std::string rqData; //needs to be locked with mpd_mutex

		//needed for threading
		std::thread* m_thread;
		std::atomic<bool> stop;		

		std::string getInfo(std::string const& command);
		inline void requestStatus();
		inline void parseStatus(std::string const& output);
};

static void thread_Mpd_readLoop(Mpd* mpd);


#endif // MPD
