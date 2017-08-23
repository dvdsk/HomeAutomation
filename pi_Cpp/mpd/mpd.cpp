#include "mpd.h"

#ifdef DEBUG
#define db(x) std::cerr << x;
#else
#define db(x)
#endif

static inline void error(const char *msg)
{
  perror(msg);
  exit(0);
}

//re-connect the socket to the remote server
int Mpd::reconnect(){
	//TODO check if socket connected
	if(reconn_m.try_lock() == -1){
		std::cout<<"*********RECONNECTING TO SOCKET!!!!!**************\n";
		close(sockfd);
		sockfd = socket(AF_INET, SOCK_STREAM, 0);
		if (connect(sockfd,(struct sockaddr *) &serv_addr,sizeof(serv_addr)) < 0)
			error("ERROR connecting");
		}
	return sockfd;
}

void Mpd::safeWrite(int sockfd, const char* message, int len){
	int BUFFERSIZE = 100;	
	uint8_t buffer[BUFFERSIZE];

	if(write(sockfd, message, len) == -1){
		std::cout<<"redone write\n";
		write(reconnect(), message, len); //reconnect returns the sockfd
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
	std::cout<<"TESTING\n";
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

	pollfd pollsocketfd; //used for storing socket status info
	pollsocketfd.events = POLLIN;
	pollsocketfd.fd = sockfd;

	uint8_t n;
	std::string buffer2 = "";
	std::string output;
	const char* idle = "idle\n";
	const char* status = "status\n";

	int rv;
	{	
		std::lock_guard<std::mutex> guard(mpd_mutex);
		safeWrite(sockfd,status,strlen(status));	
		safeWrite(sockfd,idle,strlen(idle));	
	}

	std::cout<<"mpd watcher started\n";
	while(!stop){
		if(rv = poll(&pollsocketfd, 1, 100) != 0){; //wait 100 milisec for data
			if(pollsocketfd.revents & POLLIN){
				n = read(pollsocketfd.fd, buffer, BUFFERSIZE);
				if(n == -1){std::cout<<"\033[1;31mREAD ERROR\033[0m\n"; while(1);}
				if(n == 0){
					std::cout<<"remote host has closed the connection, re-establishing\n";
					pollsocketfd.fd = reconnect();		
				}
				else{
					buffer2.append(buffer, n);		
					bzero(buffer, BUFFERSIZE); //TODO //FIXME n should work too

					db("read from socket: "<<std::to_string(n)<<"\n")
					//db(buffer2<<"\n\n")
					while((loc = buffer2.find("OK\n") ) != std::string::npos){
						if(loc > 3)
							processMessage(buffer2.substr(0, loc));
						buffer2.erase(0, loc+3);
					}
				}
			}
		}
		else if (rv == -1)
    	perror("poll"); // error occurred in poll()
	}
	db("done readLoop\n")
}

//TODO check const etc
void Mpd::processMessage(std::string output){
	std::lock_guard<std::mutex> guard2(dataRQ_m); //TODO FIXME needed?
	std::cout<<"1: got lock\n";
	db("started processMessage\n")
	//check if notification from server
	if(output.substr(0,8) == "changed:")
		requestStatus();
	//check if status message
	else if(output.substr(0,7) == "volume:")
		parseStatus(output);
	//check if version string
	else if(output.substr(0, sizeof("OK MPD version")) == "OK MPD version:")
		std::cout<<"NEW CONNECTION SUCCESSFULLY ESTABLISHED\n";
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
	}
	{
		//std::lock_guard<std::mutex> guard2(dataRQ_m);
		//std::cout<<"\033[1;34mlocked dataRQ_m\033[0m\n";

		dataReqested = true;
	}
	{
	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,command.c_str(),strlen(command.c_str()));
	}
	std::cout<<"2: data requested\n";

	//get data from read thread
	//no need for lock around data as access is controlled by cv and 
	//mpd_mutex already.
	cv.wait(lk, [this](){return dataRdy;});
	std::cout<<"got notified\n";
	dataRdy = false;
	info = rqData;

	{
	std::lock_guard<std::mutex> guard(mpd_mutex);
	safeWrite(sockfd,startIdle,strlen(startIdle));
	}



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
		//std::cout<<"inWhile\n"; 
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
		//std::cout<<"inWhile\n"; 
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
//	
//	std::cin.ignore();

//	mpd->saveAndClearCP();	
//	std::cout<<"\033[1;34mdone saveAndClear\033[0m\n";

//	mpd->QueueFromPLs("calm", 3*60, 5*60);
//	std::cout<<"\033[1;34mdone queue1\033[0m\n";
//	mpd->QueueFromPLs("energetic", 10*60, 11*60);
//	std::cout<<"\033[1;34mdone queue2\033[0m\n";
//	mpd->QueueFromPLs("active", 30*60, 60*60);
//	std::cout<<"\033[1;34mdone queue3\033[0m\n";
//	mpd->QueueFromPLs("calm", 3*60, 5*60);
//	std::cout<<"\033[1;34mdone queue1\033[0m\n";
//	mpd->QueueFromPLs("energetic", 10*60, 11*60);
//	std::cout<<"\033[1;34mdone queue2\033[0m\n";
//	mpd->QueueFromPLs("active", 30*60, 60*60);
//	std::cout<<"\033[1;34mdone queue3\033[0m\n";
//	mpd->QueueFromPLs("calm", 3*60, 5*60);
//	std::cout<<"\033[1;34mdone queue1\033[0m\n";
//	mpd->QueueFromPLs("energetic", 10*60, 11*60);
//	std::cout<<"\033[1;34mdone queue2\033[0m\n";
//	mpd->QueueFromPLs("active", 30*60, 60*60);
//	std::cout<<"\033[1;34mdone queue3\033[0m\n";
//	mpd->QueueFromPLs("calm", 3*60, 5*60);
//	std::cout<<"\033[1;34mdone queue1\033[0m\n";
//	mpd->QueueFromPLs("energetic", 10*60, 11*60);
//	std::cout<<"\033[1;34mdone queue2\033[0m\n";
//	mpd->QueueFromPLs("active", 30*60, 60*60);
//	std::cout<<"\033[1;34mdone queue3\033[0m\n";
//	
//  return 0;
//}
