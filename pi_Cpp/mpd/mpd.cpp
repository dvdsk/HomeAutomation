#include "mpd.h"
#include <stdio.h> //debugging


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

Mpd::Mpd(MpdState* mpdState_, SignalState* signalState_){
	char buffer[256];
	int n;
	mpdState = mpdState_;
	signalState = signalState_;


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

void thread_Mpd_readLoop(std::shared_ptr<Mpd> mpd,
	   std::shared_ptr<std::atomic<bool>> notShuttingdown)
{
	mpd->readLoop(notShuttingdown);
}

void Mpd::readLoop(std::shared_ptr<std::atomic<bool>> notShuttingdown){

	char buffer[256];
	uint8_t bufferSize;
	uint8_t bufferSize_old;
	uint8_t n;
	std::string output;
	const char* idle = "idle\n";

	{	
		std::lock_guard<std::mutex> guard(mpd_mutex);
		write(sockfd,idle,strlen(idle));	
	}

	std::cout<<"mpd watcher started\n";
	while(*notShuttingdown){//TODO replace with not shutdown
		//read till ok		
		bufferSize = 0;	
		bufferSize_old =0;
		bzero(buffer,256);
		bool notDone = true;

		while(*notShuttingdown && notDone){//TODO replace with not shutdown
			n = read(sockfd,buffer+bufferSize,256);
			bufferSize += n;		

			for(bufferSize_old; bufferSize_old<bufferSize-1; bufferSize_old++)
				if(buffer[bufferSize_old] == 'O' && buffer[bufferSize_old+1] == 'K')
					notDone = false; 		
		}
		output = buffer;

		//check if notification from server
		if(output.substr(0,8) == "changed:")
			requestStatus();
		//check if status message
		else if(output.substr(0,7) == "volume:")
			parseStatus(output);
		else
			printf("%s\n",buffer);		
	}
	std::cout<<"Mpd status loop shutting down\n";
}

inline void Mpd::requestStatus(){
	const char* status = "status\n";
	const char* idle = "idle\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,status,strlen(status));	
	write(sockfd,idle,strlen(idle));	
}

inline void Mpd::parseStatus(std::string const& output){

	//parse the respons
	std::lock_guard<std::mutex> guard(mpdState->m);
	mpdState->volume = stoi(output.substr(8,2));

	if(output.substr(110,4) == "stop")
		mpdState->stopped = false;
	else if(output.substr(110,4) == "paus")
		mpdState->paused = false;
	else
		mpdState->playing = true;

	signalState->runUpdate();//always run update since there always is a change
}

void Mpd::sendCommand(std::string const& command){
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,stopIdle,strlen(stopIdle));
	write(sockfd,command.c_str(),strlen(command.c_str()));
	write(sockfd,startIdle,strlen(startIdle));
}

void Mpd::sendCommandList(std::string &command){
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";

	command = "command_list_begin\n"+command+"command_list_end\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,stopIdle,strlen(stopIdle));
	write(sockfd,command.c_str(),strlen(command.c_str() ) );
	write(sockfd,startIdle,strlen(startIdle));
}



//int main()
//{
//	std::shared_ptr<std::atomic<bool>> notShuttingdown = std::make_shared<std::atomic<bool>>();
//	*notShuttingdown = true;
//	
//	std::shared_ptr<Mpd> music = std::make_shared<Mpd>();
//	std::thread t1 (thread_readLoop, music, notShuttingdown);

//	PressEnterToContinue();
//	music->sendCommand("status\n");
//	PressEnterToContinue();

//	t1.join();
//	
//  return 0;
//}
