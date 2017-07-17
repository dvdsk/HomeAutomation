#include "minimalShell.h"

#define MAX_ARGS 50
#define MAX_PATH_SIZE 1024

/**
 * Find the full path of an executable.
 * @param execName The name of the executed program
 * @param path An array of the directories in which to look for the executable
 * @param fullpath The output of the full path of the found executable
 * @param argList An array of the arguments to the executable
 * @return -1 on error, 0 on success
 */
int findExecutable(const char* execName, const char* path[], char* fullpath){
	if (strcmp(execName, "") == 0) {
	  return -1;
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


std::string minimalShell(char* argv[], std::string input){
	constexpr int WRITE_END = 1;
	constexpr int READ_END = 0;
  int Input[2], Output[2];
	pid_t pid;

	char fullpath[100];
	char buf[100];
	int nRead;
	std::string str;

	if(findExecutable(argv[0], mypath, fullpath) == -1)
		std::cout<<"error finding executable\n"; 	

	argv[0] = fullpath;

	pipe(Input);
	pipe(Output);	
	pid = fork();

	if (pid == 0) {// We're in the child here.

		close(Input[WRITE_END]);//Close the writing end of the input pipe
		close(Output[READ_END] );//Close the reading end of the output pipe

		dup2(Input[READ_END], STDIN_FILENO);
		dup2(Output[WRITE_END], STDOUT_FILENO);
		dup2(Output[WRITE_END], STDERR_FILENO);
		
		close(Input[READ_END]); //Close fd that are no longer needed
		close(Output[WRITE_END]); //Close fd that are no longer needed
		
		execv(argv[0], argv); 	
		exit(EXIT_FAILURE);//as execv should exit on its own

	} else {// We're in the parent here.
    
    close(Input[READ_END]);// Close the reading end of the input pipe.
    close(Output[WRITE_END]);// Close the writing end of the output pipe

    // Write to child’s stdin
    write(Input[WRITE_END], input.c_str(), input.size());
    close(Input[WRITE_END]); //sends EOF

		/* read output of the subprocess */
		while((nRead = read(Output[READ_END], buf, 100)) >0){
			str.append(buf, nRead);
		}
		close(Output[READ_END] );//Close the reading end of the output pipe
  }
	return str;
}

/* example usage */
//int main() {
//	
//	char* argv[] = {"at", "now", "+", "1", "minutes", NULL};
//	minimalShell(argv, "./homeAutomation startWakeup");

//	return 0;
//}
