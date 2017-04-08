#include "plotly.h"

namespace plotly{

	PlotData::PlotData(){
	}

	//TODO rewrite using fast format to save time
	void addHttpFormated_Time(std::string &data, uint32_t x[], int len){	
		time_t rawtime;
		struct tm *timeinfo;
		char buffer[24];//was 24
		data += "x: [";
		for(int i=0; i<len; i++){
			rawtime = (time_t)x[i];
			timeinfo = localtime(&rawtime);
			strftime (buffer,24,"\'%F %T\', ",timeinfo); //2013-10-04 22:23:00 =format
			data+= buffer;
		}
		data[data.length()-2] = ']';
		data[data.length()-1] = ',';
	}

	//TODO rewrite using fast format to save time
	void addHttpFormated_float(std::string &data, float y[], int len){	
		data += "y: [";
		for(int i=0; i<len; i++){
			data += std::to_string(y[i])+", ";
		}
		data[data.length()-2] = ']';
		data[data.length()-1] = ',';
	}

	void add_trace(std::string &data, PlotData &plotDat, uint32_t x[], float y[], int len){
		std::string name = "test";

		data += "var trace1 = {";
		plotDat.traces += "trace1";

		addHttpFormated_Time(data, x, len);
		addHttpFormated_float(data, y, len);
		data += "name: '"+name+"',";
		data += "yaxis: 'y1',";
		data += "type: 'scatter'";

		data += "};";
	}

}
