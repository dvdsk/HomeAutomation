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

Mpd::~Mpd(){
	const char* stopIdle = "noidle\n";

	//request data
	std::lock_guard<std::mutex> guard(mpd_mutex);
	write(sockfd,stopIdle,strlen(stopIdle));
}

void thread_Mpd_readLoop(Mpd* mpd,
	   std::atomic<bool>* notShuttingdown)
{
	mpd->readLoop(notShuttingdown);
}

void Mpd::readLoop(std::atomic<bool>* notShuttingdown){
	
	constexpr int BUFFERSIZE = 100;
	size_t loc;
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
		n = read(sockfd, buffer, BUFFERSIZE);
		buffer2.append(buffer, n);		
		bzero(buffer,n);

		
		while((loc = buffer2.find("OK\n") ) != std::string::npos){
			processMessage(buffer2.substr(0, loc));
			buffer2.erase(0, loc+3);
			//std::cout<<"buffer now: "<<buffer2<<"\n";
		}
	}
	std::cout<<"Mpd status loop shutting down\n";
}

//TODO check const etc
void Mpd::processMessage(std::string output){

	//check if notification from server
	if(output.substr(0,8) == "changed:")
		requestStatus();
	//check if status message
	else if(output.substr(0,7) == "volume:")
		parseStatus(output);
	else if(output.size() > 3 && (output.find("file:") == 0) ){//dataReqested
		dataReqested = false;
		dataRdy = true;
		rqData = output;
		cv.notify_all();	
	}
	else debugPrint("\033[1;31mOUTPUT: "+output+" DATARQ:"+std::to_string(dataReqested)+"\033[0m\n\n");
	//std::cout<<"output: "<<output<<"<<<<<<<<\n";

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
	//std::cout<<"parsed shizzle !!!!!!\n";
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
	std::lock_guard<std::mutex> guard(mpd_mutex);
	dataReqested = true;

	write(sockfd,stopIdle,strlen(stopIdle));
	write(sockfd,command.c_str(),strlen(command.c_str()));
	write(sockfd,startIdle,strlen(startIdle));

	//get data from read thread
	//no need for lock around data as access is controlled by cv and 
	//mpd_mutex already.
	cv.wait(lk, [this](){return dataRdy;});
	dataRdy = false;
	info = rqData;

	return info;
}

void Mpd::QueueFromPLs(std::string const &source, 
	const int tMin, const int tMax){

	std::vector<int> runTimes;
	std::vector<std::string> filePaths; 
	int len=0, start, stop=0, time=0, r;
	std::string toAdd;

	//request and organise needed song data
	std::string info = getInfo("listplaylistinfo "+source+"\n");
	//debugPrint("info: "+info+" endInfo\n");

	while(1 ){ 
		start = info.find("file:", stop);
		if(start == std::string::npos){break;}
		stop = info.find("\n", start);
		filePaths.push_back( info.substr(start+6, stop-(start+6)));
		//std::cout<<info.substr(start+6, stop-(start+6))<<"\n";

		start = info.find("Time:", stop);
		stop = info.find("\n", start);
		debugPrint("SUBSTR**:"+info.substr(start+6, stop-(start+6))+"\n");
		runTimes.push_back( std::stoul( info.substr(start+6, stop-(start+6))));
		len++;
	}

	//make a random list of songs and send that to the MPD
	std::minstd_rand generator(std::time(0)); 

	while(time<tMin && len != 0){
		r = (int) (generator()%(len-1+1) +0);

		if(time+runTimes[r]<tMax){
			toAdd+="add \""+filePaths[r]+"\"\n";
			time+=runTimes[r];
		}
		filePaths[r] = filePaths[len-1];
		runTimes[r] = runTimes[len-1];
		len--;
	};

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

	mpd->QueueFromPLs("calm", 3*60, 5*60);
	//PressEnterToContinue();
	mpd->QueueFromPLs("energetic", 10*60, 11*60);

	PressEnterToContinue();

	*notShuttingdown = false;
	t1.join();
	
  return 0;
}
