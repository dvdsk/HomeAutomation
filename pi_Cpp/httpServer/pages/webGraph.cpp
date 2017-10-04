#include "webGraph.h"

std::string* WebGraph::plotly_mainPage(){
	float y[MAXPLOTRESOLUTION];
	uint32_t x[MAXPLOTRESOLUTION];
	int	len;

	std::string* page = new std::string;
	plotly::PlotData plotDat(page);

	*page ="\
	<html>\
		<head>\
		  <script src=\"https://cdn.plot.ly/plotly-latest.min.js\"></script>\
		</head>\
\
		<body>\
			<div id=\"myDiv\" style=\"width: 90vw; height: 90vh;\"/div>\
			<script>";


	uint32_t now = this_unix_timestamp();

	int secondsToPlot = 2*24*3600;
	uint32_t t1 = now-secondsToPlot;
	uint32_t t2 = now;

	secondsToPlot = -1;
	len = slowData->fetchSlowData(t1, t2, x, y, TEMP_BED);
	std::cout<<"len: "<<len<<"\n";
	plotly::add_trace(plotDat, x, y, len, plotly::TEMP, "temperature bed");

	len = slowData->fetchSlowData(t1, t2, x, y, HUMIDITY_BED);
	plotly::add_trace(plotDat, x, y, len, plotly::HUMID, "humidity bed");

	len = slowData->fetchSlowData(t1, t2, x, y, CO2PPM);
	plotly::add_trace(plotDat, x, y, len, plotly::CO2, "co2");

	len = slowData->fetchSlowData(t1, t2, x, y, BRIGHTNESS_BED);
	plotly::add_trace(plotDat, x, y, len, plotly::BRIGHTNESS, "brightness bed");

	plotly::setData(plotDat);
	plotly::setLayout(plotDat);

	*page += "\
				Plotly.newPlot('myDiv', data, layout);\
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

std::string* WebGraph::bathroomSensors(){
	float y[MAXPLOTRESOLUTION];
	uint32_t x[MAXPLOTRESOLUTION];
	int	len;

	std::string* page = new std::string;
	plotly::PlotData plotDat(page);

	*page ="\
	<html>\
		<head>\
		  <script src=\"https://cdn.plot.ly/plotly-latest.min.js\"></script>\
		</head>\
\
		<body>\
			<div id=\"myDiv\" style=\"width: 90vw; height: 90vh;\"/div>\
			<script>";


	uint32_t now = this_unix_timestamp();

	int secondsToPlot = 1*24*3600; //1 day
	uint32_t t1 = now-secondsToPlot;
	uint32_t t2 = now;

	len = slowData->fetchSlowData(t1, t2, x, y, TEMP_BATHROOM);
	std::cout<<"len: "<<len<<"\n";
	plotly::add_trace(plotDat, x, y, len, plotly::TEMP, "temperature bed");

	len = slowData->fetchSlowData(t1, t2, x, y, HUMIDITY_BATHROOM);
	plotly::add_trace(plotDat, x, y, len, plotly::HUMID, "humidity bed");

	plotly::setData(plotDat);
	plotly::setLayout(plotDat);

	*page += "\
				Plotly.newPlot('myDiv', data, layout);\
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

	page+= dy_getData(toPlot, now-5*24*3600, now);

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


WebGraph::WebGraph(PirData* pirData_, SlowData* slowData_){
	dyCss = load_file("sources/dygraph.css");
	dyjs = load_file("sources/dygraph.min.js");

	pirData = pirData_;
	slowData = slowData_;

  //check if key could be read
  if ((dyCss == nullptr) || (dyjs == nullptr))
  {
    printf ("The dyCss/dyjs files should be in sources/dygraph.css and sources/dygraph.min.js\n");
  }
}

WebGraph::~WebGraph(){
	if(dyCss != nullptr) delete[] dyCss;
	if(dyjs != nullptr) delete[] dyjs;


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
//used to load the key files into memory
char* WebGraph::load_file(const char* filename) {
  FILE *fp;
  char* buffer;
  unsigned long size;

  size = get_file_size(filename);
  if (0 == size)
    return nullptr;

  fp = fopen(filename, "rb");
  if (!fp)
    return nullptr;

  buffer = new char[size + 1];
  if (!buffer) {
      fclose (fp);
      return nullptr;
  }
  buffer[size] = '\0';

  if (size != fread(buffer, 1, size, fp)) {
      free(buffer);
      buffer = nullptr;
  }

  fclose(fp);
  return buffer;
}

uint32_t WebGraph::this_unix_timestamp() {
	time_t t = std::time(0);
	uint32_t now = static_cast<uint32_t> (t);
	return now;
}
