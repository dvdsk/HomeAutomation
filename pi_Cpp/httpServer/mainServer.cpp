#include "mainServer.h"

int print_out_key (void *cls, enum MHD_ValueKind kind, 
                   const char *key, const char *value)
{
  printf ("%s: %s\n", key, value);
  return MHD_YES;
}

inline int authorised_connection(struct MHD_Connection* connection){
	int fail;
	char* pass = NULL;
	char* user = MHD_basic_auth_get_username_password(connection, &pass);
	fail = ( (user == NULL) ||
				 (0 != strcmp (user, config::HTTPSERVER_USER)) ||
				 (0 != strcmp (pass, config::HTTPSERVER_PASS)) );  
	if (user != NULL) free (user);
	if (pass != NULL) free (pass);
	return fail;
}

inline void convert_arguments(void* cls, TelegramBot*& bot, MainState*& state){
	void** arrayOfPointers;
	void* element1;
	void* element2;

	//convert arguments back (hope this optimises well),
	//this gives us access to all the classes below with all threads
	//having the same functions and variables availible.
	arrayOfPointers = (void**)cls;
	element1 = (void*)*(arrayOfPointers+0);
	element2 = (void*)*(arrayOfPointers+1);
	bot = (TelegramBot*)element1;
	state = (MainState*)element2;
	return;																		 
}

int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
		                     const char* method, const char* version, const char* upload_data,
		                     size_t* upload_data_size, void** con_cls) {
  int ret;  
  int fail;
  struct MHD_Response *response;	

	TelegramBot* bot;
	MainState* state;

	convert_arguments(cls, bot, state);
 
	fail = authorised_connection(connection); //check if other party authorised
  if (0 == strcmp(method, "GET")){
		if (NULL == *con_cls) {*con_cls = connection; return MHD_YES;}
		
		printf ("New %s request for %s using version %s\n", method, url, version);
		

		
		//if user authentication fails
		if (fail){
				const char* page = "<html><body>Go away.</body></html>";
				response = MHD_create_response_from_buffer(strlen (page), (void*) page, 
									 MHD_RESPMEM_PERSISTENT);
				ret = MHD_queue_basic_auth_fail_response(connection, "my realm",response);
			}
		//continue with correct response if authentication is successfull
		else{
				const char *page = "<html><body>A secret.</body></html>";
				
				state->httpSwitcher(url); //can send commands to state
				
				response = MHD_create_response_from_buffer(strlen (page), (void *) page, 
									 MHD_RESPMEM_PERSISTENT);
				ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
			}
  }//else possible telegram webhook call
  else if (0 == strcmp(method, "POST")){
		
		bot->processMessage(); //entire bot hangs on this function 
		
	}
	//unrecognised or unallowed protocol break connection
	else{ return MHD_NO;}
  
  MHD_destroy_response (response);
  return ret;
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
//FIXME was static and not used wanted to get rid of warning
char* load_file (const char *filename)
{
  FILE *fp;
  char* buffer;
  unsigned long size;

  size = get_file_size(filename);
  if (0 == size)
    return NULL;

  fp = fopen(filename, "rb");
  if (! fp)
    return NULL;

  buffer = (char*)malloc(size + 1);
  if (! buffer)
    {
      fclose (fp);
      return NULL;
    }
  buffer[size] = '\0';

  if (size != fread (buffer, 1, size, fp))
    {
      free (buffer);
      buffer = NULL;
    }

  fclose (fp);
  return buffer;
}


int thread_Https_serv(std::shared_ptr<std::mutex> stop, 
											std::shared_ptr<TelegramBot> bot,
											std::shared_ptr<MainState> state){
												
  struct MHD_Daemon* daemon;
  char *key_pem;
  char *cert_pem;
  
  key_pem = load_file("server.key");
  cert_pem = load_file("server.pem");

  //check if key could be read
  if ((key_pem == NULL) || (cert_pem == NULL))
  {
    printf ("The key/certificate files could not be read.\n");
    return 1;
  }

	//make an array of pointers used to pass through to the
	//awnser to connection function (the default handler). This array
	//is read only. The pointers better be in shared memory space for 
	//the awnser function to be able to reach them
	void* arrayOfPointers[2] = {bot.get(), state.get()};



  daemon = MHD_start_daemon (MHD_USE_SELECT_INTERNALLY | MHD_USE_SSL,
														 config::HTTPSERVER_PORT, NULL, NULL,
                             &answer_to_connection, (void*)arrayOfPointers,
                             MHD_OPTION_HTTPS_MEM_KEY, key_pem,
                             MHD_OPTION_HTTPS_MEM_CERT, cert_pem,
                             MHD_OPTION_END);
  
  //check if the server started alright                           
  if(NULL == daemon)
    {
      printf ("%s\n", cert_pem);
      //free memory if the server crashed
      free(key_pem);
      free(cert_pem);

      return 1;
    }
    
	//for as long as we cant lock stop we keep the server
	//up. Stop is the shutdown signal.
	(*stop).lock();	
	std::cout<<"shutting https server down gracefully\n";
	
  //free memory if the server stops
  MHD_stop_daemon(daemon);
	std::cout<<"HOI\n";
  free(key_pem);
  free(cert_pem);	
	(*stop).unlock();	  
  return 0;      
}
