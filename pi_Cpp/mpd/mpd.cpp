#include "mpd.h"

void error(const char *msg)
{
    perror(msg);
    exit(0);
}

Mpd::Mpd(){
	//create TCP internet socket
	sockfd = socket(AF_INET, SOCK_STREAM, 0);
  if (sockfd < 0) 
      error("ERROR opening socket");	

	//get host info  
	server = gethostbyname(hostname);
  if (server == NULL) {
      fprintf(stderr,"ERROR, no such host\n");
      exit(0);
  }
	//copy host info, protocol and port into struct socketaddressinfo (sockaddr_in)
  bzero((char *) &serv_addr, sizeof(serv_addr));
  serv_addr.sin_family = AF_INET;
  bcopy((char *)server->h_addr, (char *)&serv_addr.sin_addr.s_addr, server->h_length);
	//htons converts values between host and network byte order
  serv_addr.sin_port = htons(portno);	

	//and finally connect the socket to the remote server
  if (connect(sockfd,(struct sockaddr *) &serv_addr,sizeof(serv_addr)) < 0) 
  	error("ERROR connecting");

	//check if connected to mpd and empty socket
	bzero(buffer,256);
	n = read(sockfd,buffer,255);
	if(strcmp(buffer, "OK MPD") > 6){std::cout<<"Connected to MPD succesfully\n";}
}

void Mpd::pause(){
	const char* command = "pause 1\n";
	write(sockfd,command,strlen(command));

	bzero(buffer,256);
	n = read(sockfd,buffer,255);
  printf("%s",buffer);
}

void Mpd::resume(){
	const char* command = "pause 0\n";
	write(sockfd,command,strlen(command));

	bzero(buffer,256);
	n = read(sockfd,buffer,255);
  printf("%s",buffer);
}

void Mpd::idle(){
	const char* command = "idle player mixer\n";
	write(sockfd,command,strlen(command));

	bzero(buffer,256);
	n = read(sockfd,buffer,255);
  printf("%s\n",buffer);
}

int main()
{
	Mpd mpd;
	//mpd.pause();
	//mpd.resume();
	mpd.idle();

  return 0;
}
