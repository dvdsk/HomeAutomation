#include "mpd.h"
#include <stdio.h> //debugging
#include <thread> //debugging


void PressEnterToContinue()
  {
  int c;
  printf( "Press ENTER to continue... " );
  fflush( stdout );
  do c = getchar(); while ((c != '\n') && (c != EOF));
  }

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

//void Mpd::pause(){
//	const char* command = "pause 1\n";
//	write(sockfd,command,strlen(command));

//	bzero(buffer,256);
//	n = read(sockfd,buffer,255);
//  printf("%s",buffer);
//}

//void Mpd::resume(){
//	const char* command = "pause 0\n";
//	write(sockfd,command,strlen(command));

//	bzero(buffer,256);
//	n = read(sockfd,buffer,255);
//  printf("%s",buffer);
//}

void Mpd::parseStatus(){
	bool playing;
	int volume;

	const char* command = "status\n";
	write(sockfd,command,strlen(command));	

	bzero(buffer,256);
	n = read(sockfd,buffer,255);

	//parse the respons
	std::string output(buffer);
	volume = stoi(output.substr(8,2));
	if(output.substr(110,4) == "stop"){playing = false;}
	else{playing = true;}
}

//const char* startIdle = "idle player mixer\n";
//const char* stopIdle = "noidle\n";
//write(sockfd,stopIdle,strlen(stopIdle));
//write(sockfd,command,strlen(command));
//write(sockfd,startIdle,strlen(startIdle));

void statusLoop(int sockfd, std::shared_ptr<std::atomic<bool>> notShuttingdown){

	char* buffer2[256];
	char* buffer3[256];
	uint8_t bufferSize;
	uint8_t bufferSize_old;
	uint8_t n;

	std::cout<<"PLEASE LOOK AT ME\n";
	std::cout<<"buffer3[2]: "<<buffer3[1]<<"\n";
	std::cout<<"PLEEEAAASE\n";

	while(*notShuttingdown){//TODO replace with not shutdown
		//read till ok		
		bufferSize = 0;	
		bufferSize_old =0;
		bzero(buffer2,256);		

		while(*notShuttingdown){//TODO replace with not shutdown
			std::cout<<"alive2\n";
			n = read(sockfd,buffer2,10);
			bufferSize += n;		

//			std::cout<<"buffer[10]: "<<+buffer2[10]<<"\n";
//			std::cout<<"bufferSize: "<<+bufferSize<<"\n";
			for(bufferSize_old; bufferSize_old<bufferSize-1; bufferSize_old++){
//				std::cout<<+buffer2[bufferSize_old]<<"-"<<bufferSize_old+1<<"\n";
				if(buffer2[bufferSize_old] == "O" && buffer2[bufferSize_old+1] == "K")
					break;
			}
		}
		std::cout<<"PROCESSED STUFF\n";
		printf("%s\n",buffer2);		
		//cut the data into strings on completion codes ("OK")
		//and process those strings 
	}
	std::cout<<"Mpd status loop shutting down\n";
}

void Mpd::idle(){
	const char* command = "idle player mixer\n";
	write(sockfd,command,strlen(command));

	bzero(buffer,256);
	n = read(sockfd,buffer,255);
  printf("%s\n",buffer);

	PressEnterToContinue();

	bzero(buffer,256);
	n = read(sockfd,buffer,255);
  printf("%s\n",buffer);
}

int main()
{
	std::shared_ptr<std::atomic<bool>> notShuttingdown = std::make_shared<std::atomic<bool>>();
	*notShuttingdown = true;
	
	Mpd mpd;
	//mpd.pause();
	//mpd.resume();
	
	std::thread t1 (statusLoop, mpd.sockfd, notShuttingdown);
	//statusLoop(mpd.sockfd, notShuttingdown);

//	std::cout<<"alive3\n";
//	const char* command = "status\n";
//	write(mpd.sockfd,command,strlen(command));	

	t1.join();
//	void idle();
	
  return 0;
}
