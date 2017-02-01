#include "lamps.h"

/*
void getUsername(){
	CURL *curl;
  CURLcode res;
 
  // get a curl handle  
  curl = curl_easy_init();
  if(curl) {
    curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api").c_str() );
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, requestID);
 
    // Perform the request, res will get the return code 
    res = curl_easy_perform(curl);
    // Check for errors  
    if(res != CURLE_OK)
      fprintf(stderr, "curl_easy_perform() failed: %s\n",
              curl_easy_strerror(res));
 
    // always cleanup 
    
    std::cout<<res<<"\n";
    
    curl_easy_cleanup(curl);
  }
  curl_global_cleanup();
}*/

size_t readCurlToString(void *contents, size_t size, 
																	size_t nmemb, std::string *s){
    size_t newLength = size*nmemb;
    size_t oldLength = s->size();
    try
    {
        s->resize(oldLength + newLength);
    }
    catch(std::bad_alloc &e)
    {
        //handle memory problem
        return 0;
    }

    std::copy((char*)contents,(char*)contents+newLength,s->begin()+oldLength);
    return size*nmemb;
}

static size_t read_callback(void *src, size_t size, size_t nmemb, void *stream){
  int retcode = 0;
  int strSize;
	
	std::cout<<"now: "<<(const char*)stream<<"\n";
	strSize = strlen((const char*)stream);

  std::memcpy(src, stream, strSize);
	std::cout<<(const char*)src<<"\n";
	std::cout<<strSize<<"\n";
 
  return strSize;
}

LampsAPI::LampsAPI(){
	curl = curl_easy_init();
	return;
}

LampsAPI::~LampsAPI(){
	curl_easy_cleanup(curl);
	return;
}

std::string LampsAPI::post(std::string apiCall, std::string post){
	std::string result;
  CURLcode res;
	
	curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api/"+username+apiCall).c_str() );
	curl_easy_setopt(curl, CURLOPT_POSTFIELDS, post.c_str() );
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	res = curl_easy_perform(curl);
	std::cout<<res<<"\n";
	if(res != CURLE_OK)
      fprintf(stderr, "curl_easy_perform() failed: %s\n",
              curl_easy_strerror(res));
	return result;
}

std::string LampsAPI::get(std::string apiCall){
	std::string result;

	curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api/"+username+apiCall).c_str() );
	curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L); //no need for https
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	curl_easy_perform(curl);
	return result;
}

std::string LampsAPI::put(std::string apiCall, std::string toput){
  CURLcode res;
	std::string result;
	const char* strToPut = toput.c_str();
	
	std::cout<<"here: "<<strToPut<<"\n";
	
	curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api/"+username+apiCall).c_str() );
	curl_easy_setopt(curl, CURLOPT_UPLOAD, 1L);
	//curl_easy_setopt(curl, CURLOPT_PUT, 1L);

	curl_easy_setopt(curl, CURLOPT_READFUNCTION, read_callback);
	curl_easy_setopt(curl, CURLOPT_READDATA, strToPut);
	curl_easy_setopt(curl, CURLOPT_INFILESIZE_LARGE,
									 (curl_off_t)strlen(strToPut));
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	res = curl_easy_perform(curl);
	std::cout<<res<<"\n";
	if(res != CURLE_OK)
      fprintf(stderr, "curl_easy_perform() failed: %s\n",
              curl_easy_strerror(res));

	return result;	
}

void LampsAPI::allOff(){
	post("/light/1/state","{\"on\": false}");
}

//returns a string with all lights
void LampsAPI::getLights(){
	std::cout<<get("/lights");	
}

int main(void)
{
	LampsAPI hue;
	hue.allOff();
	hue.getLights();
  return 0;
}
