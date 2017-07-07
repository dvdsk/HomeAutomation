#ifndef MAINSERVER
#define MAINSERVER

#include <microhttpd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <mutex>
#include <memory>

#include <iostream>

#include "../state/mainState.h"
#include "../telegramBot/telegramBot.h"
#include "pages/webGraph.h"
#include "../config.h"

//following this tutorial:
//https://www.gnu.org/software/libmicrohttpd/tutorial.html

//used by load_file to find out the file size
//FIXME was static and not used wanted to get rid of warning
long get_file_size (const char *filename);

//used to load the key files into memory
//FIXME was static and not used wanted to get rid of warning
char* load_file (const char *filename);

inline void convert_arguments(void* cls, TelegramBot*& bot, HttpState*& httpState, 
	SignalState*& signalState, WebGraph*& webGraph);

inline int authorised_connection(struct MHD_Connection* connection);

int thread_Https_serv(std::mutex* stop, 
											TelegramBot* bot,
											HttpState* httpState,
											SignalState* signalState,
											PirData* pirData,
											SlowData* slowData);
												 
int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
												 const char* method, const char* version, const char* upload_data,
												 size_t* upload_data_size, void** con_cls);

int print_out_key (void *cls, enum MHD_ValueKind kind, 
									 const char *key, const char *value);



#endif

