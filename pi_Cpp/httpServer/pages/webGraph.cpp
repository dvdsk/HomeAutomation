#include "webGraph.h"

const char* WebGraph::mainPage(){
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
									columns: ";
	uint32_t now = this_unix_timestamp();
	std::vector<plotables> toPlot;
	toPlot.push_back(TEMP_BED);
	page+= getData(toPlot, now-60*60, now);
	page+="				}\
						});\
				</script>\
			</body>\
		</html>";
	return page.c_str();
}

std::string WebGraph::getData(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT){
	
	float y[MAXPLOTRESOLUTION];
	uint32_t x[MAXPLOTRESOLUTION];
	int len;
	bool gotx = false;

	std::string data;
	data.reserve(toPlot.size()*MAXPLOTRESOLUTION);//allocate extra data

	for(int i=0; i<toPlot.size(); i++){
		switch(toPlot[i]){
      case TEMP_BED:
        {										         
					len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; toHttpFormat_Time(data, x, len);}
					toHttpFormat_Temp(data, "temperature bed", y, len); 
        }
        break;
      case HUMIDITY_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; toHttpFormat_Time(data, x, len);}
					toHttpFormat_Temp(data, "Humidity bed", y, len); 
        }
        break;
      case CO2PPM:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; toHttpFormat_Time(data, x, len);}
					toHttpFormat_Temp(data, "Co2", y, len); 
        }
        break;
      case BRIGHTNESS_BED:
        {
          len = slowData->fetchSlowData(startT, stopT, x, y, toPlot[i]);//todo
					if(!gotx){gotx = true; toHttpFormat_Time(data, x, len);}
					toHttpFormat_Temp(data, "Brightness bed", y, len); 
        }
        break;    
      default:
        break;
    }
  }
}

//TODO rewrite using fast format to save time
void WebGraph::toHttpFormat_Time(std::string &data, uint32_t x[], int len){	
	data += "['x'";
	for(int i=0; i<len; i++){
		data+","+std::to_string(x[i]);
	}
	data+= "], ";
}

//TODO rewrite using fast format to save time
void WebGraph::toHttpFormat_Temp(std::string &data, const char* legend_name, float y[], int len){	
	data = data+ "['"+legend_name+"'";
	for(int i=0; i<len; i++){
		data = data+","+std::to_string(y[i]);
	}
	data+= "], ";
}

WebGraph::WebGraph(std::shared_ptr<PirData> pirData_, std::shared_ptr<SlowData> slowData_){
	C3css = load_file("sources/c3.css");
	C3js = load_file("sources/c3.js");

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
