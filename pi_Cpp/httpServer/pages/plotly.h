#ifndef PLOTLY
#define PLOTLY

#include <string>

namespace plotly{

	enum Axes { TEMP, HUMID, CO2, BRIGHTNESS };


	class PlotData {
		public:
		PlotData(std::string* httpStr_);
		std::string* httpStr;
		std::string traces;

		int nAxis;
		int nLines;
		std::string layout;
	};	

	void addHttpFormated_Time(std::string &data, uint32_t x[], int len);     
	void addHttpFormated_float(std::string &data, float x[], int len);

	void add_trace(PlotData &plotDat, uint32_t x[], float y[], int len, Axes axis, std::string title);
	void setData(PlotData &plotDat);
	void setLayout(PlotData &plotDat);
}

#endif
