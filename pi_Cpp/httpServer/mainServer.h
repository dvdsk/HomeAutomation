#ifndef MAINSERVER
#define MAINSERVER

#include <microhttpd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <mutex>

#include <iostream>

#include "../state/mainState.cpp"

//following this tutorial:
//https://www.gnu.org/software/libmicrohttpd/tutorial.html

constexpr int PORT = 8888;

//used by load_file to find out the file size
//FIXME was static and not used wanted to get rid of warning
long get_file_size (const char *filename);

//used to load the key files into memory
//FIXME was static and not used wanted to get rid of warning
char* load_file (const char *filename);

int th_Https_serv(std::shared_ptr<std::mutex> stop);


//typedef void(*f2_type)(signed short int sensorData[9], byte PIRs[2], 
//byte rqUpdate1[1], byte rqUpdate2[1]);
int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
												 const char* method, const char* version, const char* upload_data,
												 size_t* upload_data_size, void** con_cls);

int print_out_key (void *cls, enum MHD_ValueKind kind, 
									 const char *key, const char *value);



#endif

