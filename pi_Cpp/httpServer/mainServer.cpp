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

//used by load_file to find out the file size
static long get_file_size (const char *filename)
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
static char* load_file (const char *filename)
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

int main() {
 
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

  daemon = MHD_start_daemon (MHD_USE_SELECT_INTERNALLY | MHD_USE_SSL,
	   		                     PORT, NULL, NULL,
                             &answer_to_connection, NULL,
                             MHD_OPTION_HTTPS_MEM_KEY, key_pem,
                             MHD_OPTION_HTTPS_MEM_CERT, cert_pem,
                             MHD_OPTION_END);
  
  //check if the server started alright                           
  if(NULL == daemon)
    {
      printf ("%s\n", cert_pem);
      //free memory if the server crashed
      free (key_pem);
      free (cert_pem);

      return 1;
    }  
  
  std::cout<<"HELLO?\n";
  getchar ();

  //free memory if the server stops
  MHD_stop_daemon (daemon);
  free (key_pem);
  free (cert_pem);

  return 0;
}


