#include "createStringHeader.h"

std::string input;
std::string output = "";
int i = 0;

void ProcessedHtml::handleComment(){
	db("handeling comment") 
	//enterd comment
	while(strncmp(str+i-2, "-->", 3) != 0)	i++;
	//when comment is done skip to next non linefeed/space
	i++;
	while(str[i] == ' ' || str[i] == '\n' || str[i] == '\r' || str[i] == '\t') i++;	
	i--;
}

void ProcessedHtml::copyChar(){
	output += str[i];
	i++;
}

void ProcessedHtml::copyChar(int n){
	for(int j =0; j < n; j++) {
		output += str[i];
		i++;
	}
}

void ProcessedHtml::ignoreChar(){
	i++;
}

void ProcessedHtml::ignoreChar(int n){
	for(int j =0; j < n; j++)
		i++;
}

void ProcessedHtml::takeKeyWord(){
	//copy keyword and space after it
	db("taking keyword") 
	while(i < strSize){
		if(str[i] == ' '){
			copyChar();
			return;
		}
		else if(str[i] == '>')
			return;
		else{
			copyChar();
		}
	}
	return;
}

void ProcessedHtml::handleString(){
	//in a string
	db("handeling string") 
	copyChar();
	while(i < strSize){
		if(str[i] == '"'){
			copyChar();
			break;
		}
		else
			copyChar();
	}
	db("done handeling string") 
}

void ProcessedHtml::handleString_JS(){
	//in a string
	db("handeling string") 
	copyChar();
	while(i < strSize){
		if(str[i] == '\''){
			copyChar();
			break;
		}
		else
			copyChar();
	}
	db("done handeling string") 
}

void ProcessedHtml::handleJs(){
	db("handeling Javascript")
	copyChar(8);
	
	while(i < strSize) {
		if(strncmp(str+i, "script>", 7) == 0){
			copyChar(6);
			break;		
		}
		else if(strncmp(str+i, "var", 3) == 0)
			copyChar(4);
		else if(str[i] == '\t' || str[i] == ' '	|| str[i] == '\n')
			ignoreChar();
		else if(str[i] == '\'')
			handleString_JS();
		else
			copyChar();
	}
}

void ProcessedHtml::handleSplit(){
	StringVariable strVar;

	strVar.posInStr = output.length();
	ignoreChar(strlen(split_start));
	
	while(strncmp(str+i, split_stop, strlen(split_stop)) != 0 && i <strSize){
		strVar.varName += str[i];
		ignoreChar();
	}
	std::cout<<"adding: "<<strVar.varName<<"\n";
	splitpoints.push_back(strVar);
	ignoreChar(strlen(split_stop));
}

void ProcessedHtml::process(){

	while(i < strSize) {
		//strip comments
		if(strncmp(str+i, "<!--", 4) == 0){
			handleComment();
		}
		//decrease < > part size
		else if(str[i] == '<'){
			if(strncmp(str+i+1, "script", 6) == 0) {
				handleJs();				
			} else {			
				takeKeyWord();
				//delete all spaces not in string
				while(i < strSize-1){
					//in a string copy everything
					if(str[i] == '>')
						break;
					else if(str[i] == ' ') 
						ignoreChar();
					else if(str[i] == '"')
						handleString();
					else if(strncmp(str+i, split_start, strlen(split_start)) == 0)
						handleSplit();
					else
						copyChar();
				}
			}
			copyChar();
		}
		//keep the character
		else if(str[i] == ' ' || str[i] == '\n' || str[i] == '\t')
			ignoreChar();
		else if(strncmp(str+i, split_start, strlen(split_start)) == 0)
			handleSplit();
		else
			copyChar();			
	}
}

ProcessedHtml::ProcessedHtml(std::string &input)
: output(input)
{
	strSize = input.size();
	str = new char[strSize];
	memcpy(str, input.c_str(), strSize);
	currentSplitPoint = 0;
	i = 0;

	output.clear(); //clear the string without deallocating mem
	process();
}

std::string& ProcessedHtml::getMinified(){
	return output;
}

StringVariable* ProcessedHtml::getNextSplitPoint(){
	StringVariable* strVar;
	if(currentSplitPoint < splitpoints.size()){
		strVar = &splitpoints[currentSplitPoint];
		currentSplitPoint++;
		return strVar;
	} else
		return nullptr;
}

void processFile(std::string filename, std::ofstream& hFile){
  std::ifstream inFile(filename.c_str());
	
	//read in the entire file
	if(inFile.is_open()){
    // get length of file:
    inFile.seekg(0, std::ios::end);
    input.resize(inFile.tellg());
    inFile.seekg(0, std::ios::beg);

		inFile.read(&input[0], input.size());
		inFile.close();
	}
	ProcessedHtml processed(input);		
	int n = 0; 
	StringVariable* stringVar;
	hFile<<"namespace "<<filename.substr(0, filename.length()-5)<<"{\n";
	while((stringVar = processed.getNextSplitPoint()) != nullptr){
		hFile<<"constexpr const char* "<<stringVar->varName<<" = R\"delimiter(";
		hFile.write(processed.getMinified().c_str()+n, stringVar->posInStr-n);
		hFile<<")delimiter\";\n";
		n = stringVar->posInStr;
	};
	int len = processed.getMinified().length();
	hFile<<"constexpr const char* LAST = R\"delimiter(";
	hFile.write(processed.getMinified().c_str()+n, len-n);
	hFile<<")delimiter\";\n"<<"}\n";
	
}

int main(int argc, char** argv) {
	std::ofstream hFile("webStrings.h", std::ofstream::out | std::ofstream::trunc);
	
	processFile("dashboard.html", hFile);
}
