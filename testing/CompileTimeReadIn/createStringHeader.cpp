#include <iostream>
#include <fstream>
#include <string>
#include <regex>

//std::string stripHTML(std::string str){
//	//strip comments
//	//std::regex std_rx(R"((?<=>)\s*)");
//	std::regex remove_comments( R"((<!--)[\s\S]*(-->))" );

//	//strip comments
//	std::string output = regex_replace(str, remove_comments, "");
//	return output;
//}

//std::string stripHTML(std::string input){
//	std::string output;
//	output.resize(input.size());
//	unsigned int strSize = input.size();

//	const char* str = input.c_str();

//	//iterate over input
//	output += str[0];
//	output += str[1];
//	output += str[2];
//	for(unsigned int i = 3; i < strSize; ++i) {
//		//in a string copy everything
//		if(str[i] == '"'){ 
//			//in a string
//			output += str[i]; i++;
//			while(str[i] != '"'){
//				output += str[i]; i++;
//			}
//			output += str[i]; //i++ is handled in the for loop
//		}
//		//strip comments
//		else if(str[i-3] == '<' && str[i-2] == '!' && str[i-1] == '-' && str[i] == '-'){
//			//remove the comment signs already put in output str
//			output.pop_back(); 
//			output.pop_back(); 
//			output.pop_back();
//			i+=3;
//			//enterd comment
//			while(!(str[i-2] == '-' && str[i-1] == '-' && str[i] == '>'))	i++;
//			//when comment is done skip to next non linefeed/space
//			i++;
//			while(str[i] == ' ' || str[i] == '\n' || str[i] == '\r' || str[i] == '\t') i++;
//			output += str[i];	
//		}
//		else if(str[i] == '>'){
//			output += str[i];
//			i++;
//			while(str[i] == ' ' || str[i] == '\n' || str[i] == '\r' || str[i] == '\t') i++;
//			output += str[i];			
//		}
//		//keep the character
//		else{
//			output += str[i];
//		}
//	}
//	return output;
//}

std::string stripHTML(std::string input){
	std::string output;
	output.resize(input.size());
	unsigned int strSize = input.size();

	const char* str = input.c_str();

//	//iterate over input
//	output += str[0];
//	output += str[1];
//	output += str[2];
	int i = 0;
	while(i < strSize) {
		std::cout<<"str[i] "<<str[i]<<"\n";
		//strip comments
		if(strncmp(str+i, "<!--", 4) == 0){
			//enterd comment
			while(strncmp(str+i-2, "-->", 3) != 0)	i++;
			//when comment is done skip to next non linefeed/space
			i++;
			while(str[i] == ' ' || str[i] == '\n' || str[i] == '\r' || str[i] == '\t') i++;	
			i--;
			std::cout<<str[i]<<"\n";
		}
		//decrease < > part size
		else if(str[i] == '<'){
			std::cout<<"hi\n";
			//copy keyword 
			while(str[i] != ' '){
				output += str[i];
				i++;
			}
			i++;//copy first space after keyword
			//delete all spaces not in " " 
			while(str[i] != '>'){
				//in a string copy everything
				if(str[i] == ' ') i++;
				else if(str[i] == '"'){ 
					//in a string
					output += str[i]; i++;
					while(str[i] != '"'){
						output += str[i]; i++;
					}
					output += str[i]; i++;
				}
				else{
					output += str[i]; i++;	
				}
			}
			output += str[i];
			std::cout<<"output: "<<str[i+1]<<"\n";
		}
//		else if(str[i] == ' '){

//		}

		//keep the character
		else if(str[i] != ' '){
			output += str[i];
		}
	i++;
	}
	return output;
}

int main(int argc, char** argv) {
  std::string contents;
  std::ifstream file("example.html");
	
	//read in the entire file
	if(file.is_open()){
    // get length of file:
    file.seekg(0, std::ios::end);
    contents.resize(file.tellg());
    file.seekg(0, std::ios::beg);

		file.read(&contents[0], contents.size());
		file.close();
	}
	contents = stripHTML(contents);
	std::cout<<contents<<"\n";
}
