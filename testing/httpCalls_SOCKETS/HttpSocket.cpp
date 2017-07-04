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

HttpSocket::HttpSocket(const char* host, uint16_t port){
  /* first what are we going to send and where are we going to send it? */
  int portno =        80;
  //char *host =        "api.somesite.com";
  char *message_fmt = "POST /apikey=%s&command=%s HTTP/1.0\r\n\r\n";

  struct hostent *server;
  int bytes, sent, received, total;
  char message[1024],response[4096];

  //if (argc < 3) { puts("Parameters: <apikey> <command>"); exit(0); }

  /* fill in the parameters */
  //sprintf(message,message_fmt,argv[1],argv[2]);
  //printf("Request:\n%s\n",message);

	memcpy(message, "GET / HTTP/1.0\r\n\r\n", strlen("GET / HTTP/1.0\r\n\r\n"));

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
  char message[1024],response[4096];
	memcpy(message, "GET / HTTP/1.0\r\n\r\n", strlen("GET / HTTP/1.0\r\n\r\n"));

  /* send the request */
  total = strlen(message);
  sent = 0;
  do {
    bytes = write(sockfd,message+sent,total-sent);
    if (bytes < 0)
        error("ERROR writing message to socket");
    if (bytes == 0)
        break;
    sent+=bytes;
  } while (sent < total);

  /* receive the response */
  memset(response,0,sizeof(response));
  total = sizeof(response)-1;
  received = 0;
  do {
    bytes = read(sockfd,response+received,total-received);
    if (bytes < 0)
        error("ERROR reading response from socket");
    if (bytes == 0)
        break;
    received+=bytes;
  } while (received < total);

  if (received == total)
      error("ERROR storing complete response from socket");

  /* process response */
  printf("Response:\n%s\n",response);

//	if(connect(sockfd, (struct sockaddr*)&addr, sizeof(struct sockaddr_in)) == -1){
//			std::cerr<<"could not connect to socket";
//			return;
//	}

//	std::lock_guard<std::mutex> guard(httpSocket_mutex);
//	write(sockfd,request.c_str(),request.size());	
}


int main()
{
	
	constexpr int BUFFER_SIZE = 1024;
	char buffer[BUFFER_SIZE];

	HttpSocket* lampServ = new HttpSocket("example.com", 80);

	std::this_thread::sleep_for(std::chrono::seconds(10));

	lampServ->send("GET /\r\n");

	std::this_thread::sleep_for(std::chrono::minutes(10));
	std::cout<<"AFTER 10 MIN:\n";
	lampServ->send("GET /\r\n");


//	bzero(buffer, BUFFER_SIZE);
//	while(read(lampServ->sockfd, buffer, BUFFER_SIZE - 1) != 0){
//		fprintf(stderr, "%s", buffer);
//		bzero(buffer, BUFFER_SIZE);
//	}
	
	delete lampServ;
  return 0;
}
