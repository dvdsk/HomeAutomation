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
#include "../smallFunct/minimalShell.h"

//following this tutorial:
//https://www.gnu.org/software/libmicrohttpd/tutorial.html

enum Connectiontype {POST, GET};
enum PostKey {MINTILLALARM};

struct connection_info_struct
{
  Connectiontype connectiontype;
	PostKey postKey;
  char* answerstring;
  struct MHD_PostProcessor* postprocessor;
	char* data;
};

/* used by load_file to find out the file size */
long get_file_size (const char *filename);

/* used to load the key files into memory */
char* load_file (const char *filename);

inline int authorised_connection(struct MHD_Connection* connection);
												 
int answer_to_connection(void* cls,struct MHD_Connection* connection, const char* url,
												 const char* method, const char* version, const char* upload_data,
												 size_t* upload_data_size, void** con_cls);

/* called to process post request data */
static int iterate_post(void *coninfo_cls, enum MHD_ValueKind kind, const char *key,
												const char *filename, const char *content_type,
												const char *transfer_encoding, const char *data, 
												uint64_t off, size_t size);

/* cleans up memory used by post call */
void request_completed(void *cls, struct MHD_Connection *connection, 
     		        			 void **con_cls, enum MHD_RequestTerminationCode toe);

int print_out_key (void *cls, enum MHD_ValueKind kind, 
									 const char *key, const char *value);

int thread_Https_serv(std::mutex* stop, 
											TelegramBot* bot,
											HttpState* httpState,
											SignalState* signalState,
											PirData* pirData,
											SlowData* slowData);

inline void convert_arguments(void* cls, TelegramBot*& bot, HttpState*& httpState, 
	SignalState*& signalState, WebGraph*& webGraph);

#endif

