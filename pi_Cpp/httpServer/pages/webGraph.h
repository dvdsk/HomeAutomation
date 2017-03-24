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
	const char* mainPage();

	char* C3css;
	char* C3js;

	private:
	long get_file_size (const char *filename);
	char* load_file (const char *filename);

	void toHttpFormat_Time(std::string &data, uint32_t x[], int len);     
	void toHttpFormat_Temp(std::string &data, const char* legend_name, float x[], int len);
	uint32_t this_unix_timestamp();

	std::string getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT);

	std::shared_ptr<PirData> pirData;
	std::shared_ptr<SlowData> slowData;
};

const char* createGraphPage();

#endif

