#include "httpGetPostPut.h"

size_t readCurlToString(void *contents, size_t size, size_t nmemb, std::string *s){
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

//FIXME was static and not used wanted to get rid of warning
size_t read_callback(void *src, size_t size, size_t nmemb, void *stream){
  //int retcode = 0;
  int strSize;
	
	std::cout<<"now: "<<(const char*)stream<<"\n";
	strSize = strlen((const char*)stream);

  std::memcpy(src, stream, strSize);
	std::cout<<(const char*)src<<"\n";
	std::cout<<strSize<<"\n";
 
  return strSize;
}

HttpGetPostPut::HttpGetPostPut(std::string baseUrl_){
	baseUrl = baseUrl_;
	curl = curl_easy_init();
	return;
}

HttpGetPostPut::~HttpGetPostPut(){
	curl_easy_cleanup(curl);
	return;
}

std::string HttpGetPostPut::post(std::string url, std::string post){
	std::string result;
  CURLcode res;
	
	curl_easy_setopt(curl, CURLOPT_URL, (baseUrl+url).c_str() );
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

//std::string HttpGetPostPut::postJson(std::string url, std::string post){
	
	////create header manually;
	//struct curl_slist *headers = NULL;
	//headers = curl_slist_append(headers, "Accept: application/json");
	//headers = curl_slist_append(headers, "Content-Type: application/json");
	//headers = curl_slist_append(headers, "charsets: utf-8");
	
	//curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "POST");
	//curl_easy_setopt(curl, CURLOPT_POSTFIELDS, jsonObj);

	//curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	//curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	//res = curl_easy_perform(curl);
	//std::cout<<res<<"\n";
	//if(res != CURLE_OK)
      //fprintf(stderr, "curl_easy_perform() failed: %s\n",
              //curl_easy_strerror(res));
	//return result;
//}

//std::string HttpGetPostPut::postFile(std::string url, std::string jsonObj){
	
	////create header manually;
	//struct curl_slist *headers = NULL;
	//headers = curl_slist_append(headers, "Accept: application/json");
	//headers = curl_slist_append(headers, "Content-Type: multipart/form-data");
	//headers = curl_slist_append(headers, "charsets: utf-8");
	
	//curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "POST");
	//curl_easy_setopt(curl, CURLOPT_POSTFIELDS, jsonObj);

	//curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	//curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	//res = curl_easy_perform(curl);
	//std::cout<<res<<"\n";
	//if(res != CURLE_OK)
      //fprintf(stderr, "curl_easy_perform() failed: %s\n",
              //curl_easy_strerror(res));
	//return result;
//}



std::string HttpGetPostPut::get(std::string url){
	std::string result;

	curl_easy_setopt(curl, CURLOPT_URL, (baseUrl+url).c_str() );
	
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	curl_easy_perform(curl);
	return result;
}


//std::string HttpGetPostPut::get_noSSL(std::string url){
	//std::string result;

	//curl_easy_setopt(curl, CURLOPT_URL, (baseUrl+url).c_str() );
	//curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L); //no need for https
	
	//curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, readCurlToString);
	//curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result);
	
	//curl_easy_perform(curl);
	//return result;
//}


std::string HttpGetPostPut::putString(std::string url, std::string toput){
  CURLcode res;
	std::string result;
	const char* strToPut = toput.c_str();
	
	std::cout<<"here: "<<strToPut<<"\n";
	
	curl_easy_setopt(curl, CURLOPT_URL, (baseUrl+url).c_str() );
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
