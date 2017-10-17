#ifndef WEBGRAPH
#define WEBGRAPH

#include <microhttpd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <mutex>
#include <memory>
#include <vector>
#include <ctime>

#include <iostream>

#include "../../dataStorage/PirData.h"
#include "../../dataStorage/SlowData.h"
#include "../../config.h"
#include "plotly.h"
#include "dygraphs.h"

class WebGraph{
	public:
	WebGraph(PirData* pirData, SlowData* slowData);
	~WebGraph();
	std::string dy_mainPage(); //uses dygraphs 
	std::string* plotly_mainPage();
	std::string* bathroomSensors();
	std::string* listSensors();

	char* dyCss;
	char* dyjs;

	private:
	long get_file_size (const char *filename);
	char* load_file (const char *filename);

	uint32_t this_unix_timestamp();

	std::string dy_getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT);

	PirData* pirData;
	SlowData* slowData;
};



#endif
