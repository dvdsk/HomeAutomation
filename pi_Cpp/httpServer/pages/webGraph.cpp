#include "webGraph.h"

std::string WebGraph::plotly_mainPage(){
	float y[MAXPLOTRESOLUTION];
	uint32_t x[MAXPLOTRESOLUTION];

	std::string page ="\
	<html>\
		<head>\
		  <script src=\"https://cdn.plot.ly/plotly-latest.min.js\"></script>\
		</head>\
\
		<body>\
			<div id=\"myDiv\" style=\"width: 90vw; height: 90vh;\"/div>\
			<script>\
				var data = [{";

	uint32_t now = this_unix_timestamp();

	int	len = slowData->fetchSlowData(now-24*3600, now, x, y, TEMP_BED);

	plotly_toHttpFormat_Time(page, x, len);
	plotly_toHttpFormat_Temp(page, y, len);	

	page += "\
					type: 'scatter'\
				}];\
				Plotly.newPlot('myDiv', data);\
\
				window.onresize = function reSize(){\
					var update = {\
						width: document.getElementById('myDiv').clientWidth,\
						height: document.getElementById('myDiv').clientHeight\
					};\
					Plotly.relayout('myDiv', update);\
				};\
\
			</script>\
		</body>\
	</html>";

	return page;
}

//TODO rewrite using fast format to save time
void WebGraph::plotly_toHttpFormat_Time(std::string &data, uint32_t x[], int len){	
	std::cout<<"STARTED FORMATTING\n";
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
void WebGraph::plotly_toHttpFormat_Temp(std::string &data, float y[], int len){	
	data += "y: [";
	for(int i=0; i<len; i++){
		data += std::to_string(y[i])+", ";
	}
	data[data.length()-2] = ']';
	data[data.length()-1] = ',';
}


std::string WebGraph::C3_mainPage(){

	std::string page =
	"<html>\
		<head>\
		  <link rel=\"stylesheet\" type=\"text/css\" href=\"/css/c3.css\">\
		</head>\
			<body>\
				<div id=\"chart\"></div>\
				<script src=\"https://d3js.org/d3.v3.min.js\" charset=\"utf-8\"></script>\
				<script src=\"/js/c3.js\"></script>\
					<script>\
						var chart = c3.generate({\
								bindto: '#chart',\
								data: {\
									x: 'x',\
									columns: ";
	uint32_t now = this_unix_timestamp();
	std::vector<plotables> toPlot;
	toPlot.push_back(TEMP_BED);
	page+= C3_getData(toPlot, now-24*3600, now);
	// https://github.com/mbostock/d3/wiki/Time-Formatting#wiki-format
	page += "},\
					 axis: {\
							 x: {\
									 type: 'timeseries',\
									 tick: {\
											format: '%Y-%m-%d'\
								 	 }\
							}\
			 			}\
					});\
				</script>\
			</body>\
		</html>";

	return page;
}

std::string WebGraph::dy_mainPage(){

	uint32_t now = this_unix_timestamp();
	std::vector<plotables> toPlot;
	toPlot.push_back(TEMP_BED);

	std::string page ="\
<html>\
<head>\
<script type=\"text/javascript\"\
  src=\"dygraph.js\"></script>\
<link rel=\"stylesheet\" src=\"dygraph.css\" />\
</head>\
<body>\
<div id=\"graphdiv2\" style=\"width: 90vw; height: 90vh;\"/div>\
<script type=\"text/javascript\">\
  g2 = new Dygraph(document.getElementById(\"graphdiv2\"),";

	page+= dy_getData(toPlot, now-24*3600, now);

	page+="\
</script>\
</body>\
</html>";

	return page;
}

std::string WebGraph::dy_getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT){
	
	float y[4][MAXPLOTRESOLUTION];//TODO increase y rows with more possible plot values
	uint32_t x[MAXPLOTRESOLUTION];
	std::string labels[4];
	unsigned int labels_len = 0;
	unsigned int len;

	std::string data = "[";
	data.reserve(toPlot.size()*2*MAXPLOTRESOLUTION);//allocate extra data

	for(unsigned int i=0; i<toPlot.size(); i++){
		switch(toPlot[i]){
      case TEMP_BED:
        {										         
					len = slowData->fetchSlowData(startT, stopT, x, y[0], toPlot[i]);//todo
					labels[labels_len] = "temperature bed"; labels_len++;
        }
        break;
      case HUMIDITY_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y[1], toPlot[i]);//todo
					labels[labels_len] = "humidity bed"; labels_len++;
        }
        break;
      case CO2PPM:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y[2], toPlot[i]);//todo
					labels[labels_len] = "co2ppm bed"; labels_len++;
        }
        break;
      case BRIGHTNESS_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y[3], toPlot[i]);//todo
					labels[labels_len] = "brightness bed"; labels_len++;
        }
        break;    
      default:
        break;
    }
  }
	for(unsigned int i=0; i<len; i++){		
		data += "["+std::to_string(x[i])+"000";
		for(unsigned int j=0; j<toPlot.size(); j++){
			data += ","+std::to_string(y[j][i]);
		}
		data += "],";
	}
	data[data.length()-1] = ']';

	data+= ",{labels: [ \"x\"";
	for(unsigned int j=0;j<toPlot.size(); j++){
		data+= ", \""+labels[j]+"\"";
	}
	data += "]});";//,\
//  axis : {\
//    x : {\
//      valueFormatter: Dygraph.dateString_,\
//      valueParser: function(x) { return 1000*parseInt(x); },\
//      ticker: Dygraph.dateTicker\
//    }\
//  });";
	return data;
}

std::string WebGraph::C3_getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT){
	
	float y[MAXPLOTRESOLUTION];
	uint32_t x[MAXPLOTRESOLUTION];
	int len;
	bool gotx = false;

	std::string data = "[";
	data.reserve(toPlot.size()*MAXPLOTRESOLUTION);//allocate extra data

	for(unsigned int i=0; i<toPlot.size(); i++){
		switch(toPlot[i]){
      case TEMP_BED:
        {										         
					len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; C3_toHttpFormat_Time(data, x, len);}	
        }
        break;
      case HUMIDITY_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; C3_toHttpFormat_Time(data, x, len);}
					C3_toHttpFormat_Temp(data, "Humidity bed", y, len); 
        }
        break;
      case CO2PPM:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; C3_toHttpFormat_Time(data, x, len);}
					C3_toHttpFormat_Temp(data, "Co2", y, len); 
        }
        break;
      case BRIGHTNESS_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; C3_toHttpFormat_Time(data, x, len);}
					C3_toHttpFormat_Temp(data, "Brightness bed", y, len); 
        }
        break;    
      default:
        break;
    }
  }
	data[data.length()-2] = ' ';
	data[data.length()-1] = ']';
	return data;
}

//TODO rewrite using fast format to save time
void WebGraph::C3_toHttpFormat_Time(std::string &data, uint32_t x[], int len){	
	data += "['x'";
	for(int i=0; i<len; i++){
		data = data+","+std::to_string(x[i])+"000";
	}
	data+= "], ";
}

//TODO rewrite using fast format to save time
void WebGraph::C3_toHttpFormat_Temp(std::string &data, const char* legend_name, float y[], int len){	
	data = data+ "['"+legend_name+"'";
	for(int i=0; i<len; i++){
		data = data+","+std::to_string(y[i]);
	}
	data+= "], ";
}

WebGraph::WebGraph(std::shared_ptr<PirData> pirData_, std::shared_ptr<SlowData> slowData_){
	C3css = load_file("sources/c3.css");
	C3js = load_file("sources/c3.js");
	dyCss = load_file("sources/dygraph.css");
	dyjs = load_file("sources/dygraph.min.js");

	pirData = pirData_;
	slowData = slowData_;

  //check if key could be read
  if ((C3css == NULL) || (C3js == NULL))
  {
    printf ("The C3css/C3js files should be in sources/c3.css and sources/c3.js\n");
  }
}

long WebGraph::get_file_size (const char *filename)
{
  FILE *fp;

  fp = fopen (filename, "rb");
  if (fp)
    {
      long size;

      if ((0 != fseek (fp, 0, SEEK_END)) || (-1 == (size = ftell (fp))))
        size = 0;

      fclose (fp);

      return size;
    }
  else
    return 0;
}

//used to load the key files into memory
//FIXME was static and not used wanted to get rid of warning
char* WebGraph::load_file (const char *filename)
{
  FILE *fp;
  char* buffer;
  unsigned long size;

  size = get_file_size(filename);
  if (0 == size)
    return NULL;

  fp = fopen(filename, "rb");
  if (! fp)
    return NULL;

  buffer = (char*)malloc(size + 1);
  if (! buffer)
    {
      fclose (fp);
      return NULL;
    }
  buffer[size] = '\0';

  if (size != fread (buffer, 1, size, fp))
    {
      free (buffer);
      buffer = NULL;
    }

  fclose (fp);
  return buffer;
}

uint32_t WebGraph::this_unix_timestamp() {
	time_t t = std::time(0);
	uint32_t now = static_cast<uint32_t> (t);
	return now;
}
