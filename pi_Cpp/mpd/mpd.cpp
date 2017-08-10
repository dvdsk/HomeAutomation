#include "mpd.h"

#ifdef DEBUG
#define db(x) std::cerr << x;
#else
#define db(x)
#endif

//started processMessage
//started requestStatus
//write is ok
//write is ok
//done requestStatus
//done processMessage

//done sendCommandList
//done QueueFromPLs
//started QueueFromPLs
//started getInfo
//write is ok
//write is ok
//write is ok
//started processMessage
//started parseStatus

//done sendCommandList
//done QueueFromPLs
//started QueueFromPLs
//started getInfo
//write is ok
//write is ok
//write is ok
//started processMessage
//started parseStatus



static inline void error(const char *msg)
{
  perror(msg);
  exit(0);
}

void Mpd::safeWrite(int sockfd, const char* message, int len){

	if(write(sockfd, message, len) == -1){
		std::cout<<"*********COULD NOT WRITE TO SOCKET!!!!!**************\n";
		//re-connect the socket to the remote server
		close(sockfd);

		sockfd = socket(AF_INET, SOCK_STREAM, 0);
		if (connect(sockfd,(struct sockaddr *) &serv_addr,sizeof(serv_addr)) < 0){
			error("ERROR connecting");
		}
		write(sockfd, message, len);
	}
	else std::cout<<"write is ok\n";
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
  if (connect(sockfd,(struct sockaddr *) &serv_addr,sizeof(serv_addr)) < 0){
  	error("ERROR connecting");
	}

	//check if connected to mpd and empty socket
	bzero(buffer,256);
	n = read(sockfd,buffer,255);
	if(strcmp(buffer, "OK MPD") > 6){std::cout<<"Connected to MPD succesfully\n";}

	// We expect write failures to occur but we want to handle them where 
	// the error occurs rather than in a SIGPIPE handler.
	signal(SIGPIPE, SIG_IGN);

	//start mpd Read loop
	stop = false;
	m_thread = new std::thread(thread_Mpd_readLoop, this);
}

Mpd::~Mpd(){
	const char* stopIdle = "noidle\n";

	stop = true;
	std::cout<<"mpd DECONSTRUCTOR ran\n";
	//request data to force update so stop bool gets noticed
	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,stopIdle,strlen(stopIdle));
	close(sockfd);
	m_thread->join();
	delete m_thread;
}

static void thread_Mpd_readLoop(Mpd* mpd)
{
	mpd->readLoop();
}

void Mpd::readLoop(){
	db("started readLoop\n")
	constexpr int BUFFERSIZE = 100;
	size_t loc;
	char buffer[BUFFERSIZE];
	bzero(buffer,BUFFERSIZE);

	uint8_t n;
	std::string buffer2 = "";
	std::string output;
	const char* idle = "idle\n";
	const char* status = "status\n";

	{	
		std::lock_guard<std::mutex> guard(mpd_mutex);
		safeWrite(sockfd,status,strlen(status));	
		safeWrite(sockfd,idle,strlen(idle));	
	}

	std::cout<<"mpd watcher started\n";
	while(!stop){//TODO replace with not shutdown	
		n = read(sockfd, buffer, BUFFERSIZE);
		if(n == -1){std::cout<<"\033[1;31mREAD ERROR\033[0m\n"; }
		buffer2.append(buffer, n);		
		bzero(buffer,n);

		while((loc = buffer2.find("OK\n") ) != std::string::npos){
			if(loc > 3)
				processMessage(buffer2.substr(0, loc));
			buffer2.erase(0, loc+3);
		}
		db(buffer2<<"\n\n")
	}
	db("done readLoop\n")
}

//TODO check const etc
void Mpd::processMessage(std::string output){
	db("started processMessage\n")
	//check if notification from server
	if(output.substr(0,8) == "changed:")
		requestStatus();
	//check if status message
	else if(output.substr(0,7) == "volume:")
		parseStatus(output);
	//otherwise must be requested data
	else if(dataReqested ){
		std::cout<<"\033[1;35mgot rq data\033[0m\n";
		dataReqested = false;
		dataRdy = true;
		rqData = output;
		cv.notify_all();	
	}
	else debugPrint("\033[1;31mOUTPUT: "+output+" DATARQ:"+
	     std::to_string(dataReqested)+"\033[0m\n\n");
	db("done processMessage\n");
}

inline void Mpd::requestStatus(){
	db("started requestStatus\n")
	const char* status = "status\n";
	const char* idle = "idle\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,status,strlen(status));	
	safeWrite(sockfd,idle,strlen(idle));	
	db("done requestStatus\n")
}

inline void Mpd::parseStatus(std::string const& output){
	db("started parseStatus\n")
	//parse the respons
	mpdState->volume = stoi(output.substr(8,3));
	mpdState->playlistlength = stoi(output.substr(output.find("playlistlength:")+15, 4));

	if(output.rfind("stop") != std::string::npos){
		mpdState->playback = STOPPED;
	}
	else if(output.rfind("paus") != std::string::npos){
		mpdState->playback = PAUSED;
	}
	else{
		mpdState->playback = PLAYING;
	}
//	db("\033[1;32mrunning Update\033[0m\n")  //TODO FIXME
	signalState->runUpdate();//always run update since there always is a change  //TODO FIXME
	db("done parseStatus\n") 
}

void Mpd::sendCommand(std::string const& command){
	db("started sendCommand\n")
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,stopIdle,strlen(stopIdle));
	safeWrite(sockfd,command.c_str(),strlen(command.c_str()));
	safeWrite(sockfd,startIdle,strlen(startIdle));
	db("done sendCommand\n")
}

void Mpd::sendCommandList(std::string &command){
	db("started sendCommandList\n")
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";

	command = "command_list_begin\n"+command+"command_list_end\n";

	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,stopIdle,strlen(stopIdle));
	safeWrite(sockfd,command.c_str(),strlen(command.c_str() ) );
	safeWrite(sockfd,startIdle,strlen(startIdle));
	db("done sendCommandList\n")
}

std::string Mpd::getInfo(std::string const& command){
	db("started getInfo\n")
	const char* startIdle = "idle player mixer\n";
	const char* stopIdle = "noidle\n";
	std::unique_lock<std::mutex> lk(cv_m);
	std::string info;

	//request data
	{
	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,stopIdle,strlen(stopIdle));

	dataReqested = true;
	safeWrite(sockfd,command.c_str(),strlen(command.c_str()));
	safeWrite(sockfd,startIdle,strlen(startIdle));
	}

	//get data from read thread
	//no need for lock around data as access is controlled by cv and 
	//mpd_mutex already.
	cv.wait(lk, [this](){return dataRdy;});
	dataRdy = false;
	info = rqData;

	db("done getInfo\n")
	return info;
}

void Mpd::QueueFromPLs(std::string const &source, 
	const unsigned int tMin, const unsigned int tMax){
	db("started QueueFromPLs\n")

	std::vector<int> runTimes;
	std::vector<std::string> filePaths; 
	int start; //as std::string::npos = -1
	unsigned int len=0, stop=0, time=0, r;
	std::string toAdd;

	//request and organise needed song data
	std::string info = getInfo("listplaylistinfo "+source+"\n");

	while(1){
		std::cout<<"inWhile\n"; 
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

	while(time<tMin && len != 0){
		std::cout<<"inWhile\n"; 
		r = (int) (generator()%(len-1+1) +0);

		if(time+runTimes[r]<tMax){
			toAdd+="add \""+filePaths[r]+"\"\n";
			time+=runTimes[r];
		}
		filePaths[r] = filePaths[len-1];
		runTimes[r] = runTimes[len-1];
		len--;
	};

	db("about to send commandlist\n")
	sendCommandList(toAdd);
	db("done QueueFromPLs\n")
}

void Mpd::saveAndClearCP(){
	db("started saveAndClearCP\n")
	std::vector<std::string> filePaths; 
	std::string commands;
	unsigned int stop=0;
	int start;

	if(mpdState->playlistlength != 0){
		std::string info = getInfo("playlistinfo\n");

		while(1){ 
			start = info.find("file:", stop);
			if(start == std::string::npos){break;}
			stop = info.find("\n", start);
			filePaths.push_back( info.substr(start+6, stop-(start+6)));
		}
	}

	commands = "playlistclear oldPL\n";
	for(auto const& path: filePaths){
		commands+="playlistadd oldPL \""+path+"\"\n";}
	commands+="clear\n";

	sendCommandList(commands);
	db("done saveAndClearCP\n")
}

//int main()
//{
//	MpdState* mpdState = new MpdState;
//	SignalState* signalState = new SignalState;	


//	Mpd* mpd = new Mpd(mpdState, signalState);

//	PressEnterToContinue();

//	//mpd->sendCommand("playlistclear oldPL\n");	
//	mpd->QueueFromPLs("energetic", 10*60, 11*60);
//	mpd->saveAndClearCP();
//	mpd->QueueFromPLs("calm", 3*60, 5*60);
//	//PressEnterToContinue();


//	PressEnterToContinue();

//	//*notShuttingdown = false;
//	//t1.join();
//	
//  return 0;
//}
