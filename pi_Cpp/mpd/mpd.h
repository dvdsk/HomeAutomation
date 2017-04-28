#ifndef MPD
#define MPD

#include <iostream> //cout
#include <string.h> //strcmp
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

void statusLoop(int sockfd, std::shared_ptr<std::atomic<bool>> notShuttingdown);

class Mpd{
	public:
		Mpd(); //connects to mpd
		void pause();
		void resume();
		void idle();
		void parseStatus();

		int sockfd; //sockfd file discriptor
	private:
    int n; //byte counter
    struct sockaddr_in serv_addr;
    struct hostent *server;
    char buffer[256];
};

#endif // MPD
