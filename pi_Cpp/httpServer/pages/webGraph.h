#ifndef WEBGRAPH
#define WEBGRAPH

#include <microhttpd.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <mutex>
#include <memory>
#include <vector>

#include <iostream>

#include "../../dataStorage/PirData.h"
#include "../../dataStorage/SlowData.h"
#include "../../config.h"

class WebGraph{
	public:
	WebGraph(std::shared_ptr<PirData> pirData, std::shared_ptr<SlowData> slowData);
	std::string C3_mainPage();
	std::string dy_mainPage(); //uses dygraphs 
	std::string plotly_mainPage();

	char* C3css;
	char* C3js;

	char* dyCss;
	char* dyjs;

	private:
	long get_file_size (const char *filename);
	char* load_file (const char *filename);

	void C3_toHttpFormat_Time(std::string &data, uint32_t x[], int len);     
	void C3_toHttpFormat_Temp(std::string &data, const char* legend_name, float x[], int len);
	void plotly_toHttpFormat_Time(std::string &data, uint32_t x[], int len);     
	void plotly_toHttpFormat_Temp(std::string &data, float x[], int len);
	uint32_t this_unix_timestamp();

	std::string C3_getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT);
	std::string dy_getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT);

	std::shared_ptr<PirData> pirData;
	std::shared_ptr<SlowData> slowData;
};

const char* createGraphPage();

#endif

