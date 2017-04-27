#ifndef MPD
#define MPD

#include <iostream> //cout
#include <string.h> //strcmp

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
		void pause();
		void resume();
		void idle();
	private:
    int sockfd, n; //sockfd file discriptor, byte counter
    struct sockaddr_in serv_addr;
    struct hostent *server;
    char buffer[256];
};

#endif // MPD
