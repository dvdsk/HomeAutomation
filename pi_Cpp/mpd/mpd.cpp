#include "mpd.h"
#include <stdio.h> //debugging


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

Mpd::Mpd(MpdState* mpdState_, SignalState* signalState_){
	char buffer[256];
	int n;
	mpdState = mpdState_;
	signalState = signalState_;
	dataRdy = false;
	

	//arange socket connection

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

void thread_Mpd_readLoop(Mpd* mpd,
	   std::atomic<bool>* notShuttingdown)
{
	mpd->readLoop(notShuttingdown);
}

void Mpd::readLoop(std::atomic<bool>* notShuttingdown){

	constexpr int BUFFERSIZE = 100;
	char buffer[BUFFERSIZE];
	bzero(buffer,BUFFERSIZE);

	uint8_t bufferSize;
	uint8_t n;
	std::string buffer2 = "";
	std::string output;
	const char* idle = "idle\n";

	{	
		std::lock_guard<std::mutex> guard(mpd_mutex);
		write(sockfd,idle,strlen(idle));	
	}

	std::cout<<"mpd watcher started\n";
	while(*notShuttingdown){//TODO replace with not shutdown
		//read till ok		
		bool done = false;

		while(*notShuttingdown && !done){//TODO replace with not shutdown
			//std::cout<<"reading\n";			
			n = read(sockfd, buffer, BUFFERSIZE);
			buffer2.append(buffer, n);		
			bzero(buffer,n);

			if(size_t loc = buffer2.find("OK\n") != std::string::npos){
				//std::cout<<buffer2<<"\n";
				//std::cout<<"DONE JEEEEEEEJ ******************************\n";				
				//std::cout<<buffer2.size()-loc<<"\n";				
				if(buffer2.size() - loc <= 2){
					output = buffer2.substr(0, loc+1);
					buffer2.erase(0, loc+1);
					//std::cout<<"erasing\n";
					}					
				else{
					output = std::move(buffer2);
					//std::cout<<"moving\n";
					buffer2.clear();
				}
				//std::cout<<"OUTPUT: "<<output<<"\n";
				done = true;
				break;
			}
		}
		//std::cout<<"OUTPUT: "<<output<<"\n";

		//check if notification from server
		if(output.substr(0,8) == "changed:")
			requestStatus();
		//check if status message
		else if(output.substr(0,7) == "volume:")
			parseStatus(output);
		else if(output.size() > 3 && dataReqested){
			dataRdy = true;
			rqData = output;
			cv.notify_all();			
		}
		else std::cout<<"\033[1;31mOUTPUT: "<<output<<"\033[0m\n";	
	}
	std::cout<<"Mpd status loop shutting down\n";
}

inline void Mpd::requestStatus(){
	const char* status = "status\n";
	const char* idle = "idle\n";

	//std::cout<<"rq status\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,status,strlen(status));	
	write(sockfd,idle,strlen(idle));	
}

inline void Mpd::parseStatus(std::string const& output){

	//parse the respons
	mpdState->volume = stoi(output.substr(8,2));

	if(output.substr(110,4) == "stop"){
		mpdState->playback = STOPPED;
	}
	else if(output.substr(110,4) == "paus"){
		mpdState->playback = PAUSED;
	}
	else{
		mpdState->playback = PLAYING;
	}
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

std::string Mpd::getInfo(std::string const& command){
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";
	std::unique_lock<std::mutex> lk(cv_m);
	std::string info;

	//request data
	dataReqested = true;
	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,stopIdle,strlen(stopIdle));
	write(sockfd,command.c_str(),strlen(command.c_str()));
	write(sockfd,startIdle,strlen(startIdle));

	//get data from read thread
	//no need for lock around data as access is controlled by cv and 
	//mpd_mutex already.
	cv.wait(lk, [this](){return dataRdy;});
	dataReqested = false;
	dataRdy = false;
	info = rqData;

	return info;
}

void Mpd::createPLFromPLs(std::string const &name, std::string const &source, 
	const int tMin, const int tMax){

	std::vector<int> runTimes;
	std::vector<std::string> filePaths; 
	int len=0, start, stop=0, time=0, r;
	std::string toAdd;

	//request and organise needed song data
	std::string info = getInfo("listplaylistinfo "+source+"\n");
	std::cout<<info<<"\n\n\n";

	while(1 ){ 
		start = info.find("file:", stop);
		if(start == std::string::npos){break;}
		stop = info.find("\n", start);
		filePaths.push_back( info.substr(start+6, stop-(start+6)));

		start = info.find("Time:", stop);
		stop = info.find("\n", start);
		runTimes.push_back( std::stoul( info.substr(start+6, stop-(start+6))));
		len++;
	}

	//make a random list of songs and send that to the MPD
	std::minstd_rand generator(std::time(0)); 
	auto Rmax = generator.max();
	auto Rmin = generator.min();
	std::cout<<Rmax<<", "<<Rmin<<"\n";

	std::cout<<"len: "<<len<<"\n";
	std::cout<<"size: "<<filePaths.size()<<"\n";

	if(len!=0)
		do{
			r = (int) (generator()%(len-0+1) +0);
			//r = 1;
			std::cout<<"r: "<<r<<"\n";
			std::string toAdd;
			if(time+runTimes[r]<tMax){
				toAdd+"add\""+filePaths[r]+"\" ";
				time+=runTimes[r];
			}
			filePaths[r] = filePaths[len-1];
			runTimes[r] = runTimes[len-1];
			len--;
		}while(time<tMin || len != 0);

	sendCommandList(toAdd);
}


int main()
{
	std::atomic<bool>* notShuttingdown = new std::atomic<bool>();
	*notShuttingdown = true;
	MpdState* mpdState = new MpdState;
	SignalState* signalState = new SignalState;	


	Mpd* mpd = new Mpd(mpdState, signalState);
	std::thread t1 (thread_Mpd_readLoop, mpd, notShuttingdown);

	PressEnterToContinue();

//	
////	mpd->sendCommand("add \"ACDC/01 Highway To Hell.ogg\" \n");
////	mpd->sendCommand("add \"ACDC/01 Highway To Hell.ogg\" \n");
////	mpd->sendCommand("add \"ACDC/01 Highway To Hell.ogg\" \n");
////	mpd->sendCommand("add \"ACDC/01 Highway To Hell.ogg\" \n");
//	PressEnterToContinue();
	mpd->createPLFromPLs("temp", "energetic", 300, 400);

	PressEnterToContinue();

	*notShuttingdown = false;
	t1.join();
	
  return 0;
}
