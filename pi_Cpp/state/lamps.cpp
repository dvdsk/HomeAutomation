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
  //size_t size;
  curl_off_t nread;
	
	std::cout<<"now: "<<((put_data*)stream)->data<<"\n";
	std::cout<<"now: "<<((put_data*)stream)->len<<"\n";

  memcpy(src, &(((put_data*)stream)->data), ((put_data*)stream)->len);
	size_t retcode = ((put_data*)stream)->len;
	std::cout<<retcode<<"\n";
	std::cout<<(char*)src<<"\n";
 
  nread = (curl_off_t)retcode;
 
  //fprintf(stderr, "*** We read %" CURL_FORMAT_CURL_OFF_T
          //" bytes from file\n", nread);
	retcode = 0;
  return retcode;
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
	
	curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api/"+username+apiCall).c_str() );
	curl_easy_setopt(curl, CURLOPT_POSTFIELDS, post.c_str() );
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	curl_easy_perform(curl);
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
	put_data* userdata = new put_data;
	userdata->data = (char*)toput.c_str();
	userdata->len = strlen(userdata->data);
	std::cout<<"here: "<<userdata<<"\n";
	
	curl_easy_setopt(curl, CURLOPT_URL, (hueIp+"/api/"+username+apiCall).c_str() );
	curl_easy_setopt(curl, CURLOPT_UPLOAD, 1L);
	curl_easy_setopt(curl, CURLOPT_PUT, 1L);

	curl_easy_setopt(curl, CURLOPT_READFUNCTION, read_callback);
	curl_easy_setopt(curl, CURLOPT_READDATA, userdata);
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	res = curl_easy_perform(curl);
	if(res != CURLE_OK)
      fprintf(stderr, "curl_easy_perform() failed: %s\n",
              curl_easy_strerror(res));

	return result;	
}

void LampsAPI::allOff(){
	put("/light/1/state","{\"hue\": 50000,\"on\": true,\"bri\": 200}");


}

//returns a string with all lights
void LampsAPI::getLights(){
	std::cout<<get("/lights");	
}

int main(void)
{
	LampsAPI hue;
	//hue.getLights();
	hue.allOff();
	
  return 0;
}
