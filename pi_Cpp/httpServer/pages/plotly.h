#ifndef PLOTLY
#define PLOTLY

#include <string>

namespace plotly{

	class PlotData {

		public:
		PlotData();
		std::string htmlCode;
		std::string traces;
	};	

	void addHttpFormated_Time(std::string &data, uint32_t x[], int len);     
	void addHttpFormated_float(std::string &data, float x[], int len);
	void add_trace(std::string &data, PlotData &plotDat, uint32_t x[], float y[], int len);


}

#endif
