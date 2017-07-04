#include "HttpSocket.h"
#include <stdio.h> //debugging
	#include <chrono>
	#include <thread>

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

  /* connect the socket */
  if (connect(sockfd,(struct sockaddr *)&serv_addr,sizeof(serv_addr)) < 0)
		error("ERROR connecting");
}

HttpSocket::~HttpSocket(){
  /* close the socket */
  close(sockfd);
}


void HttpSocket::send(std::string request){
  int bytes, sent, received, total;
  char* response[4096];


  /* send the request */
  sent = 0;
  do {
    bytes = write(sockfd,request.c_str()+sent,request.size()-sent);
    if (bytes < 0)
        error("ERROR writing message to socket");
    if (bytes == 0)
        break;
    sent+=bytes;
  } while (sent < request.size());
	std::cout<<"send request\n";


  /* receive the response */
  memset(response,0,sizeof(response));
  total = sizeof(response)-1;
  received = 0;
  do {
		std::cout<<"waiting for response\n";
    bytes = read(sockfd,response+received,total-received);
    if (bytes < 0) std::cerr<<"ERROR reading response from socket\n";
    if (bytes == 0){std::cout<<"done reading, breaking\n\n"; break; }
    received+=bytes;
  } while (received < total);

  if (received == total) 
		std::cerr<<"ERROR storing complete response from socket\n";


  /* process response */
  printf("Response:\n%s\n",response);

	std::cout<<"\n";
}


int main()
{
	
//	HttpSocket* lampServ = new HttpSocket("192.168.1.11", 80);
	HttpSocket* lampServ = new HttpSocket("google.com", 80);

	//https://www.jmarshall.com/easy/http/#structure
	lampServ->send("GET / HTTP/1.0\n\r\n\r");

	delete lampServ;
  return 0;
}
