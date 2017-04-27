#ifndef HTTPGETPOSTPUT
#define HTTPGETPOSTPUT

#include <stdio.h> //TODO needed?
#include <curl/curl.h>
#include <iostream> //cout
#include <string.h> //strcmp
#include <cstring> //std::memcpy


//used for http get, put and post reading of respons
size_t readCurlToString(void *ptr, size_t size, size_t nmemb, std::string *s);
//used for http put reading of string
//FIXME was static and not used wanted to get rid of warning
size_t read_callback(void *src, size_t size, size_t nmemb, void *stream);

class HttpGetPostPut{
	
	public:
	HttpGetPostPut(std::string baseUrl_);
	~HttpGetPostPut();

	//http functions
	std::string post(std::string urlCall, std::string post);
	std::string get(std::string urlCall);
	std::string put(std::string urlCall, std::string put);
	std::string get_withFile(std::string url);
	
	private:
	CURL* curl;	
	std::string baseUrl;
	
};


#endif // HTTPGETPOSTPUT
