#ifndef MPD
#define MPD

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include <memory> 
#include <atomic>

//needed for sockets
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h> 

constexpr int portno = 6600;
constexpr const char* hostname = "192.168.1.10";



class Mpd{
	public:
		Mpd(); //connects to mpd
		void readLoop(std::shared_ptr<std::atomic<bool>> notShuttingdown);
		void sendCommand(std::string const& command);
		void sendCommandList(std::string &command);

	private:
		int sockfd;//sockfd file discriptor
    struct sockaddr_in serv_addr;
    struct hostent *server;
		std::mutex mpd_mutex;

		inline void requestStatus();
		inline void parseStatus(std::string const& output);
};

void thread_readLoop(std::shared_ptr<Mpd> mpd, 
	                   std::shared_ptr<std::atomic<bool>> notShuttingdown);


#endif // MPD
