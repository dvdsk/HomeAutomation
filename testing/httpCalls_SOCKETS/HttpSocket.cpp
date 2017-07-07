#include "HttpSocket.h"
#include <stdio.h> //debugging
	#include <chrono>
	#include <thread>
	#include <cstdio> //debugging
#include <errno.h>
#include <sstream>

void PressEnterToContinue()
  {
  int c;
  printf( "Press ENTER to continue... \n" );
  fflush( stdout );
  do c = getchar(); while ((c != '\n') && (c != EOF));
  }


void error(const char *msg)
{
    perror(msg);
    exit(0);
}


HttpSocket::HttpSocket(const char* host, uint16_t portno){
  struct hostent *server;

  /* create the socket */
  sockfd = socket(AF_INET, SOCK_STREAM, 0);
  if (sockfd < 0) error("ERROR opening socket");

  /* lookup the ip address */
  server = gethostbyname(host);
  if (server == NULL) error("ERROR, no such host");

  /* fill in the structure */
  memset(&serv_addr,0,sizeof(serv_addr));
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(portno);
  memcpy(&serv_addr.sin_addr.s_addr,server->h_addr,server->h_length);

}

HttpSocket::~HttpSocket(){

}


std::string HttpSocket::send(std::string request){
  int bytes, sent, received, total;
  uint8_t buffer[BUFFSIZE];
	char* startOfMessage;
	unsigned int content_length;
	
  /* connect the socket */
  if (connect(sockfd,(struct sockaddr *)&serv_addr,sizeof(serv_addr)) < 0)
		error("ERROR connecting");

  /* send the request */
  sent = 0;
 	//lock mutex to prevent conflicts if sending from multiple threads 
	std::lock_guard<std::mutex> lock(httpSocket_mutex);	
	do {
    bytes = write(sockfd,request.c_str()+sent,request.size()-sent);
    if (bytes < 0) error("ERROR writing message to socket");
    if (bytes == 0)
        break;
    sent+=bytes;
  } while (sent < request.size());

	bool fitsBuffer = readABit(buffer);
	content_length = readHeaders(buffer, startOfMessage);
	std::string response(startOfMessage);

	if(fitsBuffer) return response;
	else if(content_length != 0)
		response.resize(content_length);
	
	readRemaining(buffer, response);

	std::cout<<response<<"\n";
	return response;
}

int HttpSocket::readHeaders(uint8_t* buffer, char* &startOfMessage){
	int content_length;

	char* contentLengthLoc = strstr((char*)buffer, "Content-Length:");
	if(contentLengthLoc != nullptr) content_length = atoi(contentLengthLoc);
	else content_length = 0;

	startOfMessage = strstr((char*)buffer, "\r\n\r\n");
	if(startOfMessage == nullptr){
		startOfMessage = (char*)buffer;
		std::cerr<<"server reply does not contain a message";
	}
	
	return content_length;
}

void HttpSocket::readRemaining(uint8_t* buffer, std::string &response){
  int bytes, received, total = BUFFSIZE;
 	constexpr bool keepReading = true;
	
	do {
 		bytes = read(sockfd,buffer,total);
 		if (bytes < 0) std::cerr<<strerror(errno)<<"\n";
 		if (bytes == 0)	break;
 		response.append((char*)buffer);
 	} while (keepReading);
}

bool HttpSocket::readABit(uint8_t* buffer){
  int bytes, received, total = BUFFSIZE;
	bool small = false;

	do {
		bytes = read(sockfd,buffer+received,total-received);
		if (bytes < 0) std::cerr<<strerror(errno)<<"\n";
		if (bytes == 0){
			small = true;
			break;
		}
		received+=bytes;
	} while (received < total);

	return small;
}


int main()
{
	
	HttpSocket* lampServ = new HttpSocket("192.168.1.11", 80);
	//HttpSocket* lampServ = new HttpSocket("www.example.com", 80);

	//https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html#sec5
	//Request-Line   = Method SP Request-URI SP HTTP-Version CRLF

	std::stringstream ss;
  ss << "GET /api/ZKK0CG0rOZY3nfhQsZbIkhH0y6P92EaaR-iBlBsk HTTP/1.0\r\n"
     << "Host: 192.168.1.11\r\n"
//     << "Host: example.com\r\n"
     << "Accept: application/json\r\n"
		 << "Connection: close\r\n"
     << "\r\n\r\n";
  std::string request = ss.str();


	//lampServ->send("GET http://www.example.com/ HTTP/1.0 \r\n\r\nConnection: \"close\"\r\n");
	lampServ->send(request);
	//lampServ->send("GET http://www.example.com/ HTTP/1.0 \r\n\r\n");

	delete lampServ;
  return 0;
}
