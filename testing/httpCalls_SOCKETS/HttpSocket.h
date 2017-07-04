#ifndef HTTPSOCKET
#define HTTPSOCKET

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include <atomic>


//needed for sockets
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/tcp.h>
#include <netinet/in.h>
#include <netdb.h> 

constexpr int portno = 6600;
constexpr const char* hostname = "192.168.1.10";

class HttpSocket{
	public:
		HttpSocket(const char* host, uint16_t port);
		~HttpSocket();
		void send(std::string request);

	private:
		std::mutex httpSocket_mutex;
		int sockfd;//sockfd file discriptor
	  struct sockaddr_in serv_addr;

};

#endif // MPD
