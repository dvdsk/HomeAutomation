#ifndef HTTPSOCKET
#define HTTPSOCKET

#include <iostream> //cout
#include <string.h> //strcmp
#include <mutex>
#include <atomic>
#include <cstring> //strstr

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

constexpr int BUFFSIZE = 4096;

class HttpSocket{
	public:
		HttpSocket(const char* host, uint16_t port);
		~HttpSocket();
		std::string send(std::string request);

	private:
		std::mutex httpSocket_mutex;
		int sockfd;//sockfd file discriptor
	  struct sockaddr_in serv_addr;

		bool readABit(char* buffer);
		void readRemaining(char* buffer, std::string &response);
		int readHeaders(char* buffer, char* &startOfMessage);
};

#endif // MPD
