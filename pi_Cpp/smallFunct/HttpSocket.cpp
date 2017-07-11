#include "HttpSocket.h"
#include <stdio.h> //debugging
#include <chrono>
#include <thread>
#include <cstdio> //debugging
#include <errno.h>
#include <sstream>

static void error(const char *msg)
{
    perror(msg);
    exit(0);
}

HttpSocket::HttpSocket(const char* host_, uint16_t portno){
	host = host_;

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
	close(sockfd);
}


std::string HttpSocket::rawRequest(const std::string request){
  unsigned int bytes, sent;
  char buffer[BUFFSIZE];
	buffer[BUFFSIZE-1] = '\0';
	memset(buffer, 0, BUFFSIZE-1);
	char* startOfMessage;
	unsigned int content_length;

	httpSocket_mutex.lock();	
	/* open new socket */
	sockfd = socket(AF_INET, SOCK_STREAM, 0);
	if (sockfd < 0) error("ERROR opening socket");	
  /* connect the socket */
  if (connect(sockfd,(struct sockaddr *)&serv_addr,sizeof(serv_addr)) < 0)
		error("ERROR connecting");

  /* send the request */
  sent = 0;
 	//lock mutex to prevent conflicts if sending from multiple threads 
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

	if(fitsBuffer){httpSocket_mutex.unlock(); return response; }
	else if(content_length != 0)
		response.resize(content_length);
	readRemaining(buffer, response);

	close(sockfd);
	httpSocket_mutex.unlock();

	return response;
}

int HttpSocket::readHeaders(char* buffer, char* &startOfMessage){
	int content_length;

	char* contentLengthLoc = strstr(buffer, "Content-Length:");
	if(contentLengthLoc != nullptr) content_length = atoi(contentLengthLoc);
	else content_length = 0;

	startOfMessage = strstr(buffer, "\r\n\r\n")+sizeof("\r\n\r\n");
	if(startOfMessage == nullptr){
		startOfMessage = buffer;
		std::cerr<<"server reply does not contain a message";
	}
	
	return content_length;
}

void HttpSocket::readRemaining(char* buffer, std::string &response){
  int bytes, total = BUFFSIZE;
 	constexpr bool keepReading = true;
	
	do {
		memset(buffer, 0, BUFFSIZE);
 		bytes = read(sockfd,buffer,total);
 		if (bytes < 0) std::cerr<<strerror(errno)<<"\n";
 		if (bytes == 0)	{break;}
 		response.append(buffer, bytes);
 	} while (keepReading);
}

bool HttpSocket::readABit(char* buffer){
  int bytes, received= 0, total = BUFFSIZE-1;
	//BUFFSIZE-1 to accomodate for adding null terminator to string as we
	//might not read the full string.
	bool small = false;

	do {
		bytes = read(sockfd,buffer+received,total-received);
		if (bytes < 0) std::cerr<<strerror(errno)<<"\n";
		if (bytes == 0){
			small = true;
			buffer[BUFFSIZE] = '\0';
			break;
		}
		received+=bytes;
	} while (received < total);

	//didnt get the full string, add null terminator
	buffer[BUFFSIZE] = '\0';
	return small;
}

std::string HttpSocket::get(const std::string resource){

	std::string request =
		 "GET "+resource+" HTTP/1.0\r\n"
		+"Host: "+host+"\r\n"
		+"Accept: application/json\r\n"
		+"Connection: close\r\n"
		+"\r\n\r\n";

//	std::string respons =	rawRequest(request);
//	std::cout<<"response GET: "<<respons<<"\n";
//	std::cout<<"returning\n";
//	return respons;
	return rawRequest(request);
}

std::string HttpSocket::put(const std::string resource, const std::string toput){

	std::string request =
		 "PUT "+resource+" HTTP/1.0\r\n"
		+"Host: "+host+"\r\n"
		+"Accept: application/json\r\n"
		+"Connection: close\r\n"
		+"Content-Length: "+std::to_string(toput.size())+"\r\n"
		+"Content-Type: text/plain; charset=UTF-8\r\n"
		+"\r\n"
		+toput
		+"\r\n\r\n";

//	std::string respons =	rawRequest(request);
//	std::cout<<"response PUT: "<<respons<<"\n";
//	return respons;
	return rawRequest(request);
}

//int main()
//{
//	
//	HttpSocket* lampServ = new HttpSocket("192.168.1.11", 80);
//	//HttpSocket* lampServ = new HttpSocket("www.example.com", 80);

//	//https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html#sec5
//	//Request-Line   = Method SP Request-URI SP HTTP-Version CRLF

//	std::stringstream ss;
//  ss << "GET /api/ZKK0CG0rOZY3nfhQsZbIkhH0y6P92EaaR-iBlBsk HTTP/1.0\r\n"
//     << "Host: 192.168.1.11\r\n"
////     << "Host: example.com\r\n"
//     << "Accept: application/json\r\n"
//		 << "Connection: close\r\n"
//     << "\r\n\r\n";
//  std::string request = ss.str();


//	//lampServ->send("GET http://www.example.com/ HTTP/1.0 \r\n\r\nConnection: \"close\"\r\n");
//	//lampServ->send(request);
//	std::cout<<lampServ->send(request)<<"\n";

//	//lampServ->send("GET http://www.example.com/ HTTP/1.0 \r\n\r\n");

//	delete lampServ;
//  return 0;
//}
