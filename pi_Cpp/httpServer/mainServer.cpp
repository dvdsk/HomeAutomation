#include <microhttpd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include <iostream>

//following this tutorial:
//https://www.gnu.org/software/libmicrohttpd/tutorial.html

const int PORT = 8888;

//working
//int answer_to_connection(void* cls,struct MHD_Connection * connection, const char * url,
//		                     const char * method, const char * version, const char * upload_data,
//		                     size_t * upload_data_size, void ** ptr) {
//  
//  const char* page  = "<html><body>Hello, browser!</body></html>";
//  struct MHD_Response *response;
//  int ret;

//  response = MHD_create_response_from_buffer (strlen(page), (void*) page,
//  					                                  MHD_RESPMEM_PERSISTENT);
//  ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
//  MHD_destroy_response(response);
//  return ret;
//}

int print_out_key (void *cls, enum MHD_ValueKind kind, 
                   const char *key, const char *value)
{
  printf ("%s: %s\n", key, value);
  return MHD_YES;
}

int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
		                     const char* method, const char* version, const char* upload_data,
		                     size_t* upload_data_size, void** con_cls) {
  
  int ret;  
  char* user;
  char* pass;
  int fail;
  struct MHD_Response *response;

  if (0 != strcmp(method, "GET")) return MHD_NO;
  if (NULL == *con_cls) {*con_cls = connection; return MHD_YES;}
  
  printf ("New %s request for %s using version %s\n", method, url, version);
  
  pass = NULL;
  user = MHD_basic_auth_get_username_password(connection, &pass);
  fail = ( (user == NULL) ||
	       (0 != strcmp (user, "root")) ||
	       (0 != strcmp (pass, "pa$$w0rd") ) );  
  if (user != NULL) free (user);
  if (pass != NULL) free (pass);
  
  //if user authentication fails
  if (fail)
    {
      const char* page = "<html><body>Go away.</body></html>";
      response = MHD_create_response_from_buffer(strlen (page), (void*) page, 
				                                         MHD_RESPMEM_PERSISTENT);
      ret = MHD_queue_basic_auth_fail_response(connection, "my realm",response);
    }
  //continue with correct response if authentication is successfull
  else
    {
      const char *page = "<html><body>A secret.</body></html>";
      response = MHD_create_response_from_buffer(strlen (page), (void *) page, 
				                                         MHD_RESPMEM_PERSISTENT);
      ret = MHD_queue_response(connection, MHD_HTTP_OK, response);
    }
  MHD_destroy_response (response);
  return ret;
}

int main() {
  struct MHD_Daemon* daemon;

  daemon = MHD_start_daemon (MHD_USE_SELECT_INTERNALLY, PORT, NULL, NULL,
                             &answer_to_connection, NULL, MHD_OPTION_END);
  if (NULL == daemon) return 1;
  
  getchar();
  MHD_stop_daemon(daemon);
  return 0;
}

