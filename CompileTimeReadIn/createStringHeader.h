#include <iostream>
#include <fstream>
#include <string>
#include <regex>
#include <string.h>

//#define PRINT_DEBUGLINES
#ifdef PRINT_DEBUGLINES
#define db(x) std::cout<<(x)<<"\n";
#else
#define db(x)
#endif

constexpr const char* split_start = "RPL_";
constexpr const char* split_stop = "_RPL";

struct StringVariable {
	std::string varName;
	int posInStr; //position in the minified string
	StringVariable(){varName == "";	}
};

class ProcessedHtml{
	public:
		ProcessedHtml(std::string& input);
		std::string& getMinified();
		StringVariable* getNextSplitPoint();

	private:
		std::vector<StringVariable> splitpoints;
		unsigned int currentSplitPoint;

		unsigned int i;
		char* str;
		unsigned int strSize;
		std::string& output;

		void handleSplit();
		void handleComment();
		void copyChar();
		void copyChar(int n);
		void ignoreChar();
		void ignoreChar(int n);
		void takeKeyWord();
		void handleString();
		void handleString_JS();
		void handleJs();
		void process();
};
