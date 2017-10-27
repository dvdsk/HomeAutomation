#include "plotly.h"

namespace plotly{

	PlotData::PlotData(std::string* httpStr_){
		httpStr = httpStr_;
		nAxis = 0;
		nLines = 1;
		layout = "var layout = {\
					showlegend: true,\
					legend: {\"orientation\": \"h\"},\
					margin: {\
						l: 42,\
						r: 20,\
						b: 0,\
						t: 0,\
						pad: 0\
					},";
		traces = "var data = [";
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
			strftime(buffer,24,"\'%F %T\', ",timeinfo); //2013-10-04 22:23:00 =format
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

	void add_trace(PlotData &plotDat, uint32_t x[], float y[], int len, Axes axis, std::string title){
		std::string name = "test";

		
		*plotDat.httpStr += "var trace"+std::to_string(plotDat.nLines)+" = {";
		plotDat.traces += "trace"+std::to_string(plotDat.nLines)+",";
		plotDat.nLines++;

		addHttpFormated_Time(*plotDat.httpStr, x, len);
		addHttpFormated_float(*plotDat.httpStr, y, len);
		*plotDat.httpStr += "mode: 'lines', name: '"+title+"',";

		plotDat.nAxis++;
		if(plotDat.nAxis==1)
			plotDat.layout += "yaxis: {";
		else{
			plotDat.layout += "yaxis"+std::to_string(plotDat.nAxis)+": {overlaying: 'y',";
			*plotDat.httpStr += "yaxis: 'y"+std::to_string(plotDat.nAxis)+"',";
		}

		*plotDat.httpStr += "type: 'scatter'";
		*plotDat.httpStr += "};";

		switch(plotDat.nAxis){
			case 1:
			break;
			case 2:
			plotDat.layout += "side: 'right',";
			break;
			case 3:
			plotDat.layout += "side: 'right', position: 1,";
			break;
			case 4:
			plotDat.layout += "side: 'left', position: 0.0,";
			break;			
		}

		switch(axis){
			case TEMP:
			plotDat.layout += "title: 'temperature (deg C)'"; //TODO use utf8 deg sign
			break;
			case HUMID:
			plotDat.layout += "title: 'humidity (percent)'"; //TODO use utf8 precent sign			
			break;
			case CO2:
			plotDat.layout += "title: 'co2 (ppm)'";
			break;
			case BRIGHTNESS:
			plotDat.layout += "title: 'brightness (relative)'";
			break;
		}
		plotDat.layout += "},";
	}

	void setData(PlotData &plotDat){
		plotDat.traces.pop_back();
		*plotDat.httpStr += plotDat.traces+"];";
	}

	void setLayout(PlotData &plotDat){
		switch(plotDat.nAxis){
			case 1:
			plotDat.layout.pop_back();			
			break;
			case 2:
			plotDat.layout.pop_back();			
			break;
			case 3:
			plotDat.layout += "xaxis: {domain: [0.0, 0.9]}";
			break;
			case 4:
			plotDat.layout += "xaxis: {domain: [0.1, 0.9]}";
			break;
		}
		*plotDat.httpStr += plotDat.layout+"};";
	}
}
