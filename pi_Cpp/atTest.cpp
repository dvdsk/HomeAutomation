#include <string>
#include <ctime>
#include <cstdint> //uint16_t
#include <iostream>
#include <thread>
#include <atomic>
#include <mutex>

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>

#define MAX_ARGS 50
#define MAX_PATH_SIZE 1024

constexpr int WRITE_END = 1;
constexpr int READ_END = 0;

/**
 * Find the full path of an executable.
 * @param execName The name of the executed program
 * @param path An array of the directories in which to look for the executable
 * @param fullpath The output of the full path of the found executable
 * @param argList An array of the arguments to the executable
 * @return -1 on error, 0 on success, 1 on internal command (e.g. cd, exit)
 */
int findExecutable(const char* execName, const char* path[], char* fullpath, char* argList[]){
	if (strcmp(execName, "") == 0) {
	  return -1;
	}
	if (strcmp(execName, "exit") == 0) {
	  exit(0);
	}
	if (strcmp(execName, "cd") == 0) {
	  if (chdir(argList[1]) != 0) {
      fprintf(stderr, "Error: %s\n", strerror(errno));
      return -1;
	  }
	  return 1;
	}
	struct stat buffer;
	if (execName[0] == '.' || execName[0] == '/') { // Path relative to current directory or absolute.
	  if (stat(execName, &buffer) == 0){
      if (buffer.st_mode & S_IXUSR) {
        strcpy(fullpath, execName);
        return 0;
      }
      fprintf(stderr, "Error: no permission to execute\n");
      return 1;
	  }
	}
	int i = 0;
	char pathname[MAX_PATH_SIZE];
	while (path[i] != NULL) {
	  strcpy(pathname, path[i]);
	  strcat(pathname, execName);
	  if (stat(pathname, &buffer) == 0){
      if (buffer.st_mode & S_IXUSR) {
        strcpy(fullpath, pathname);
        return 0;
      }
      fprintf(stderr, "Error: no permission to execute\n");
      return 1;
	  }
	  i++;
	}
	return -1;
}

const char* mypath[] = {
  "/usr/local/bin/",
  "/usr/local/sbin/",
  "/usr/bin/",
  "/usr/sbin/",
  "/bin/",
  "/sbin/",
  "/usr/games/",
  "/usr/local/games/",
  NULL
};

int main(int argc, char** argv) {
	
  char buf;
	char fullpath[100];
	char* argList[7];

  int Input[2], Output[2];
	pid_t cpid;

	if(findExecutable("at", mypath, fullpath, argList) == -1)
		std::cout<<"error finding executable\n"; 

	argList[0] = fullpath;
	argList[1] = "now";
	argList[2] = "+";
	argList[3] = "5";
	argList[4] = "minutes";	
	argList[5] = NULL;	

	pipe(Input);
	pipe(Output);
	cpid = fork();
  if (cpid == 0) {// We're in the child here.

		close(Input[WRITE_END]);//Close the writing end of the input pipe
		close(Output[READ_END] );//Close the reading end of the output pipe

		dup2(Input[READ_END], STDIN_FILENO);
		dup2(Output[WRITE_END], STDOUT_FILENO);
		dup2(Output[WRITE_END], STDERR_FILENO);
		
		close(Input[READ_END]); //Close fd that are no longer needed
		close(Output[WRITE_END]); //Close fd that are no longer needed
		
		execv(fullpath, argList); 	
		exit(EXIT_FAILURE);//as execv should exit on its own

	} else {// We're in the parent here.
    
    close(Input[READ_END]);// Close the reading end of the input pipe.
    close(Output[WRITE_END]);// Close the writing end of the output pipe

    char buffer[100];
    int count;

    // Write to child’s stdin
    write(Input[WRITE_END], "echo test\n", strlen("echo test"));
    close(Input[WRITE_END]); //sends EOF

		/* read output of the subprocess */
		while(read(Output[READ_END], &buf, 1) >0)
			write(STDOUT_FILENO, &buf, 1);
		write(STDOUT_FILENO, "\n", 1);

		close(Output[READ_END] );//Close the reading end of the output pipe
    }
	return 0;
}

//system("at -f ./homeAutomation startWakeup '13:52' today");

