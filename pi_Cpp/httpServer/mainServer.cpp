#include "mainServer.h"

const char* orderRecieved = "<html><body>Order recieved.</body></html>";
const char* unknown_page = "<html><body>A secret.</body></html>";
const char* ok = "OK";

int print_out_key (void *cls, enum MHD_ValueKind kind, 
                   const char *key, const char *value)
{
  printf ("%s: %s\n", key, value);
  return MHD_YES;
}

inline int authorised_connection(struct MHD_Connection* connection){
  bool fail = true;
  char* pass = nullptr;
  char* user = MHD_basic_auth_get_username_password(connection, &pass);
  fail = ( (user == nullptr) ||
         (0 != strcmp(user, config::HTTPSERVER_USER)) ||
         (0 != strcmp(pass, config::HTTPSERVER_PASS)) );  
  free(user);
  free(pass);// cant use delete here as MHD uses malloc internally

  return !fail;
}


inline void convert_arguments(void* cls, TelegramBot*& bot, HttpState*& httpState, 
	SignalState*& signalState, WebGraph*& webGraph){
	void** arrayOfPointers;
	void* element1;
	void* element2;
	void* element3;
	void* element4;

	//convert arguments back (hope this optimises well),
	//this gives us access to all the classes below with all threads
	//having the same functions and variables availible.
	arrayOfPointers = (void**)cls;
	element1 = (void*)*(arrayOfPointers+0);
	element2 = (void*)*(arrayOfPointers+1);
	element3 = (void*)*(arrayOfPointers+2);
	element4 = (void*)*(arrayOfPointers+3);
	bot = (TelegramBot*)element1;
	httpState = (HttpState*)element2;
	signalState = (SignalState*)element3;
	webGraph = (WebGraph*)element4; 
	return;																		 
}

int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
		                     const char* method, const char* version, const char* upload_data,
		                     size_t* upload_data_size, void** con_cls) {
  int ret;
  struct MHD_Response* response;  
	struct connection_info_struct* con_info;
	std::string pageString; //TODO change plotting platforms to get rid of this
	char* page;

	TelegramBot* bot;
	HttpState* httpState;
	SignalState* signalState;
	WebGraph* webGraph;
	convert_arguments(cls, bot, httpState, signalState, webGraph);

	#ifdef DEBUG
	printf ("New %s request for %s using version %s\n", method, url, version);
	#endif

  //if start of connection, remember connection and store usefull info about
	//connection if post create post processor.
  if (NULL == *con_cls) {
		con_info = new connection_info_struct;
 		con_info->answerstring = nullptr;
		
		//set up post processor if post request
		if(0 == strcmp (method, MHD_HTTP_METHOD_POST)){
			con_info->postprocessor = MHD_create_post_processor(connection, 
			config::POSTBUFFERSIZE, iterate_post, (void *) con_info);
			con_info->connectiontype = POST;
		}
		//als geen post request set connectiontype to GET
		else con_info->connectiontype = GET;
		
		//in all cases make sure con_cls has a value 
		*con_cls = (void*)con_info;
    return MHD_YES;
  }
  
  //correct password, repond dependig on url
  if (authorised_connection(connection)){
    
    if (0 == strcmp (method, "GET")){
			std::cout<<"url: "<<url<<"\n";
      //create diffrent pages (responses) to different url's
				//if its a state switch command send it to state for processing
				if(url[1] == '|'){		
	
					httpState->m.lock();//lock to indicate new value in url
					httpState->url = url;
					httpState->updated = true;//is atomic
					signalState->runUpdate();

					response = MHD_create_response_from_buffer(strlen (unknown_page), 
					           (void *) orderRecieved, MHD_RESPMEM_PERSISTENT); 
				}
				//else request for data
				else if(0 == strcmp(url, "/dygraph.css")){
					page = webGraph->dyCss;
					response = MHD_create_response_from_buffer(strlen (page), (void *) page, 
				             MHD_RESPMEM_PERSISTENT);
				}
				else if(0 == strcmp(url, "/dygraph.js")){
					page = webGraph->dyjs;
					response = MHD_create_response_from_buffer(strlen (page), (void *) page, 
									   MHD_RESPMEM_PERSISTENT);
				}				
				else if(0 == strcmp(url, "/graph2")){
					pageString = webGraph->dy_mainPage();
					response = MHD_create_response_from_buffer(pageString.length(), 
             	       (void *) pageString.c_str(), MHD_RESPMEM_MUST_COPY);
				}
				else if(0 == strcmp(url, "/graph3")){
					pageString = *webGraph->plotly_mainPage();	
					response = MHD_create_response_from_buffer(pageString.length(), 
             	       (void *) pageString.c_str(), MHD_RESPMEM_MUST_COPY);
				}
				else if(0 == strcmp(url, "/graph4")){
					pageString = *webGraph->bathroomSensors();	
					response = MHD_create_response_from_buffer(pageString.length(), 
             	       (void *) pageString.c_str(), MHD_RESPMEM_MUST_COPY);
				}
				else if(0 == strcmp(url, "/listData")){
					pageString = *webGraph->listSensors();	
					response = MHD_create_response_from_buffer(pageString.length(), 
             	       (void *) pageString.c_str(), MHD_RESPMEM_MUST_COPY);
				}
				else{
					response = MHD_create_response_from_buffer(strlen (unknown_page), 
					           (void *) unknown_page, MHD_RESPMEM_PERSISTENT);
				}
    //prepare respons to be send to server
    ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    }
    
    else if (0 == strcmp (method, "POST")) {     
			struct connection_info_struct* con_info = (connection_info_struct*)*con_cls;

      if (*upload_data_size != 0)	{
				
        MHD_post_process(con_info->postprocessor, upload_data,
                         *upload_data_size);
        *upload_data_size = 0;
        return MHD_YES;
      }
			else if (nullptr != con_info->answerstring){
				response = MHD_create_response_from_buffer(strlen(ok),
        (void *) ok, MHD_RESPMEM_PERSISTENT);  
				ret = MHD_queue_response(connection, MHD_HTTP_OK, response); 
			}
    }
    
  //incorrect password, present go away page
  }
  else {
    const char* page = "<html><body>Incorrect password.</body></html>";  
    response = MHD_create_response_from_buffer(strlen (page), (void*) page, 
	       MHD_RESPMEM_PERSISTENT);
    ret = MHD_queue_basic_auth_fail_response(connection, "test", response);  
  }
 
  MHD_destroy_response(response); //free memory of the respons
  return ret;
}

static int iterate_post(void *coninfo_cls, enum MHD_ValueKind kind, const char *key,
												const char *filename, const char *content_type,
												const char *transfer_encoding, const char *data, 
												uint64_t off, size_t size) {

	const char* answerstring = "nice post!";
  struct connection_info_struct* con_info = (connection_info_struct*)coninfo_cls;

  if (0 == strcmp (key, "minTillAlarm")) {
    if ((size > 0) && (size <= config::MAXNAMESIZE)) {
			con_info->postKey = MINTILLALARM;
			std::cout<<"set the key\n";

			con_info->data = new char[size+1];
			memcpy(con_info->data, data, size);
			con_info->data[size] = '\0';

			con_info->answerstring = new char[sizeof(answerstring)]; 
			memcpy(con_info->answerstring, answerstring, sizeof(*answerstring));  
    } 
    else con_info->answerstring = nullptr;  
		return MHD_NO;//inform postprocessor no further call to this func are needed
  }
	else con_info->answerstring = nullptr; 
  return MHD_YES;
}


void request_completed(void *cls, struct MHD_Connection *connection, 
     		        			 void **con_cls, enum MHD_RequestTerminationCode toe) {
  struct connection_info_struct* con_info = (connection_info_struct*)*con_cls;

  if (NULL == con_info) return;
	//do cleanup for post if the con type was a post
  if (con_info->connectiontype == POST) {
		//CLEANUP POST
		switch(con_info->postKey){
			case MINTILLALARM: 
				std::cout<<"cleaning up post\n";
				int minTillAlarm = atoi(con_info->data);
				delete[] con_info->data;				
				if(minTillAlarm > 0){
					std::cout<<minTillAlarm<<"\n";
					con_info->data = (char*)std::to_string(minTillAlarm-WAKEUP_DURATION_MIN).c_str();
					std::cout<<"con_info->data: "<<con_info->data<<"\n";
					char* argv[] = { (char*)"at", (char*)"now", (char*)"+",  
													 con_info->data, (char*)"minutes", nullptr};
					std::cout<<"shell response: "<<minimalShell(argv, "./homeAutomation startWakeup")<<"\n";
				}	
			break;
		}		
    MHD_destroy_post_processor (con_info->postprocessor);        
    delete[] con_info->answerstring;
  }
  delete con_info;
  *con_cls = NULL;   
}

//used by load_file to find out the file size
//FIXME was static 
long get_file_size (const char *filename)
{
  FILE *fp;

  fp = fopen (filename, "rb");
  if (fp)
    {
      long size;

      if ((0 != fseek (fp, 0, SEEK_END)) || (-1 == (size = ftell (fp))))
        size = 0;

      fclose (fp);

      return size;
    }
  else
    return 0;
}

//used to load the key files into memory
char* load_file(const char* filename) {
  FILE *fp;
  char* buffer;
  unsigned long size;

  size = get_file_size(filename);
  if (0 == size)
    return nullptr;

  fp = fopen(filename, "rb");
  if (!fp)
    return nullptr;

  buffer = new char[size + 1];
  if (!buffer) {
      fclose (fp);
      return nullptr;
  }
  buffer[size] = '\0';

  if (size != fread(buffer, 1, size, fp)) {
      free(buffer);
      buffer = nullptr;
  }

  fclose(fp);
  return buffer;
}



int thread_Https_serv(std::mutex* stop, 
											TelegramBot* bot,
											HttpState* httpState,
											SignalState* signalState,
											PirData* pirData,
											SlowData* slowData){
												
  struct MHD_Daemon* daemon;
  char *key_pem, *cert_pem;

  key_pem = load_file("privkey1.pem");
	cert_pem = load_file("fullchain1.pem");

  //check if key could be read
  if ((key_pem == NULL) || (cert_pem == NULL))
  {
		std::cout<<"\033[1;31mThe key/certificate files could not be read\033[0m\n";
    return 1;
  }

	//create a shared memory space for webpage functions
		std::shared_ptr<WebGraph> webGraph = std::make_shared<WebGraph>(pirData, slowData);


	//make an array of pointers used to pass through to the
	//awnser to connection function (the default handler). This array
	//is read only. The pointers better be in shared memory space for 
	//the awnser function to be able to reach them
	void* arrayOfPointers[4] = {bot, httpState, signalState, webGraph.get()};



  daemon = MHD_start_daemon (MHD_USE_SELECT_INTERNALLY | MHD_USE_SSL | MHD_USE_DEBUG,
														 config::HTTPSERVER_PORT, NULL, NULL,
                             &answer_to_connection, (void*)arrayOfPointers,
														 MHD_OPTION_NOTIFY_COMPLETED, &request_completed, NULL,
                             MHD_OPTION_HTTPS_MEM_KEY, key_pem,
                             MHD_OPTION_HTTPS_MEM_CERT, cert_pem,
                             MHD_OPTION_END);
  
  //check if the server started alright                           
  if(NULL == daemon)
    {
			std::cout<<"\033[1;31mserver could not be started, cleaning up\033[0m\n"
			         <<"\t-check if there is another server running\n";
      //free memory if the server crashed
			delete[] key_pem;
			delete[] cert_pem;	

      return 1;
    }
    
	//for as long as we cant lock stop we keep the server
	//up. Stop is the shutdown signal.
	(*stop).lock();	
	std::cout<<"shutting https server down gracefully\n";
	
  //free memory if the server stops
  MHD_stop_daemon(daemon);
  delete[] key_pem;
  delete[] cert_pem;	
	(*stop).unlock();	  
  return 0;      
}
