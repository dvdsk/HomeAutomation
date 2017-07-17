#ifndef HTTPSSOCKET
#define HTTPSSOCKET

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

//needed for openssl
#include <openssl/ssl.h>

class HttpsSocket{
	public:
		HttpsSocket(const char* host_, uint16_t port);
		~HttpsSocket();
		std::string rawRequest(const std::string request);
		std::string get(const std::string resource);
		std::string put(const std::string resource, const std::string toput);

	private:
		std::mutex httpSocket_mutex;
		int sockfd;//sockfd file discriptor
	  struct sockaddr_in serv_addr;
  	struct hostent* server;
		const char* host;
		SSL_CTX* ssl_ctx;
		SSL* conn;

		bool readABit(char* buffer);
		void readRemaining(char* buffer, std::string &response);
		int readHeaders(char* buffer, char* &startOfMessage);
};

#endif // HTTPSOCKET
