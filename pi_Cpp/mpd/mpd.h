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

		void readLoop(std::atomic<bool>* notShuttingdown);

		void sendCommand(std::string const &command);
		void sendCommandList(std::string &command);

		/* adds songs from the source playlist to the current playlist / queue 
		   a minimum and maximum runtime can be passed along */ 
		void createPLFromPLs(std::string const &name, std::string const &source,
		const int tMin, const int tMax);

	private:
		MpdState* mpdState;
		SignalState* signalState;

		int sockfd;//sockfd file discriptor
    struct sockaddr_in serv_addr;
    struct hostent *server;
		std::mutex mpd_mutex;

		std::mutex cv_m;
		std::condition_variable cv;
		bool dataRdy;
		std::atomic<bool> dataReqested;
		std::string rqData; //needs to be locked with mpd_mutex

		std::string getInfo(std::string const& command);
		inline void requestStatus();
		inline void parseStatus(std::string const& output);
};

void thread_Mpd_readLoop(Mpd* mpd, std::atomic<bool>* notShuttingdown);


#endif // MPD
