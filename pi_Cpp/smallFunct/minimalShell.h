#ifndef MINIMALSHELL
#define MINIMALSHELL

#include <string>
#include <cstring>
#include <iostream>

#include <sys/stat.h> //for stat struct
#include <unistd.h>

/* takes an char* array ending with a nullptr that with the first element
	 the shell function that needs to be executed and the other elements the
	 arguments (arguments are all the space seperated "words" in a normal terminal
	 (pieces enclosed with "" are seen as one argument)). 

	 The input is enterd into the program when it has started. The output of the
	 programm (both stdout and stderr) is passed back via a string */
std::string minimalShell(char* argv[], std::string input);

#endif

