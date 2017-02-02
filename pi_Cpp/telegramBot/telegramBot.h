#ifndef TELEGRAMBOT
#define TELEGRAMBOT

#include <stdio.h> //TODO needed?
#include <curl/curl.h>
#include <iostream> //cout
#include <string.h> //strcmp
#include <cstring> //std::memcpy
#include <memory>

#include "httpGetPostPut.h"

class TelegramBot : public HttpGetPostPut
{
	public:
		TelegramBot();
		void processMessage();
		void setWebHook();
		void test();	
	private:
		bool authorised();
		
	
		std::shared_ptr<bool> testv;
		
};


#endif // MAINSTATE
