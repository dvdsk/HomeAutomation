#ifndef LAMPS
#define LAMPS

#include <stdio.h> //TODO needed?
#include <curl/curl.h>
#include <iostream> //cout
#include <string.h> //strcmp

std::string hueIp = "192.168.1.11";

const char* requestID = "{\"devicetype\": \"my_hue_app#iphone peter\"}";
const char* username= "i5b0A56Cq754Ge6RWy8bR0xmTdD2MEwRU2jTjbjG";

//used for http get and post
size_t readCurlToString(void *ptr, size_t size, size_t nmemb, std::string *s);


struct put_data
{
  char *data;
  size_t len;
};

class LampsAPI{
	
	public:
	LampsAPI();
	~LampsAPI();

	//api functions
	void getLights(); //to be moved to private
	void allOff();
		
	private:
	CURL* curl;	
	
	//http functions
	std::string post(std::string apiCall, std::string post);
	std::string get(std::string apiCall);
	std::string put(std::string apiCall, std::string put);

	
};


#endif // MAINSTATE
