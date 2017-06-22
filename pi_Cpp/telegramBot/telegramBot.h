#ifndef TELEGRAMBOT
#define TELEGRAMBOT

#include <stdio.h> //TODO needed?
#include <curl/curl.h>
#include <iostream> //cout
#include <string.h> //strcmp
#include <cstring> //std::memcpy
#include <memory>

#include "httpGetPostPut.h"

class TelegramBot// : public HttpGetPostPut
{
	public:
		TelegramBot();// need to pass state to this so we can access its functions
									// such as parse command
		void processMessage();
		void setWebHook();
	private:
		bool authorised();
		
	
		bool testv;
		
};


#endif // MAINSTATE
