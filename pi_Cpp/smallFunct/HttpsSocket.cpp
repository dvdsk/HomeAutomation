#include "HttpsSocket.h"
#include <stdio.h> //debugging
#include <chrono>
#include <thread>
#include <cstdio> //debugging
#include <errno.h>
#include <sstream>

constexpr int BUFFSIZE = 4096;

static void error(const char *msg)
{
    perror(msg);
    exit(0);
}

HttpsSocket::HttpsSocket(const char* host_, uint16_t portno){
	host = host_;

  /* lookup the ip address */
  server = gethostbyname(host);
  if (server == NULL) error("ERROR, no such host");

  /* fill in the structure */
  memset(&serv_addr,0,sizeof(serv_addr));
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(portno);
  memcpy(&serv_addr.sin_addr.s_addr,server->h_addr,server->h_length);

	/* initialize OpenSSL */
	SSL_load_error_strings();
	SSL_library_init();
	ssl_ctx = SSL_CTX_new(SSLv23_client_method() );
}

HttpsSocket::~HttpsSocket(){
	close(sockfd);
}


std::string HttpsSocket::rawRequest(const std::string request){
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

	/* create an SSL connection and attach it to the socket */
	conn = SSL_new(ssl_ctx);
	SSL_set_fd(conn, sockfd);


	/* perform the SSL/TLS handshake with the server - when on the
	   server side, this would use SSL_accept() */
	int err = SSL_connect(conn);
	if (err != 1)
		 abort(); // handle error

  /* send the request */
  sent = 0;
 	//lock mutex to prevent conflicts if sending from multiple threads 
	do {
    bytes = SSL_write(conn,request.c_str()+sent,request.size()-sent);
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

	SSL_shutdown(conn);
	SSL_free(conn);
	close(sockfd);
	httpSocket_mutex.unlock();

	return response;
}

int HttpsSocket::readHeaders(char* buffer, char* &startOfMessage){
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

void HttpsSocket::readRemaining(char* buffer, std::string &response){
  int bytes, total = BUFFSIZE;
 	constexpr bool keepReading = true;
	
	do {
		memset(buffer, 0, BUFFSIZE);
 		bytes = SSL_read(conn,buffer,total);
 		if (bytes < 0) std::cerr<<strerror(errno)<<"\n";
 		if (bytes == 0)	{break;}
 		response.append(buffer, bytes);
 	} while (keepReading);
}

bool HttpsSocket::readABit(char* buffer){
  int bytes, received= 0, total = BUFFSIZE-1;
	//BUFFSIZE-1 to accomodate for adding null terminator to string as we
	//might not read the full string.
	bool small = false;

	do {
		bytes = SSL_read(conn,buffer+received,total-received);
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

std::string HttpsSocket::get(const std::string resource){

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

std::string HttpsSocket::put(const std::string resource, const std::string toput){

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
//	HttpsSocket* test = new HttpsSocket("www.example.com", 443);

//	std::cout<<test->get("/")<<"\n";

//	delete test;
//  return 0;
//}
