#ifndef SLOWDATA_H
#define SLOWDATA_H
#include <iostream>
#include <cstring> //memcopy, memset
#include <cstdint> //uint16_t
#include <array>
#include <fstream>      // std::fstream
#include <atomic>

#include "../config.h"
#include "../encodingScheme.h"
#include "../compression.h"
#include "MainData.h"

/*
slow data storage. Raw data is recieved from serial as a bitstream
and processed by this class. When slow data is recieved it saves and sends
on for decision making. When fast light dat is recieved it is binned and saved
only when slow data also arrives. fast data is however directly send on for 
decision making

bitstream format:
Tu = Temperature under bed, Td = Temperature on the closet at the door
Tb = Temperature in the bathroom, Hu, Hd, Hb same but then Humidity

[Tu, Td, Tb, Hu, Hd, Hb, Co2] = 70 bits. As we recieve it in bytes we ignore 
the final 2 bytes.


package format:
Temp: 9 bits        [storing -10.0 to 40.0 degrees, in values 0 to 500,
                    values 501 means lower then -10.0 and 502 higher then 40.0]]
Humidity: 10 bits   [storing 0.0 to 100.0 percent, in values 0 to 1000]
Co2: 13 bits        [storing 0 to 6000ppm, in values 0 to 6000]

We save in the same format as the bitstream

*/


//data specific functions and variables, inherits AllData
class SlowData : public Data
{
  public:    
    SlowData(const std::string filePath, uint8_t* cache, const int cacheLen);
    /*take the raw data from serial with the timestamp, rewrite it, send 
      it off for reacting if something changed and store it in a file*/
    void process(const uint8_t raw[9], const uint32_t Tstamp);

		/*fetches maxplotresolution evenly spaced data points between startT and stopT
		  if there are more then maxplotresolution elements, performs binning and leaves
		  out evenly spaced points*/
    int fetchSlowData(uint32_t startT, uint32_t stopT, 
                      uint32_t x[], float y[], plotables sensor);

		/*fetches and writes data to a file called SlowData.txt, no binning nor letting out
		  data points is performed*/
		void exportAllSlowData(uint32_t startT, uint32_t stopT);

		void preProcess_light(std::atomic<int> lightValues[], const uint32_t Tstamp);

  private:
    bool newData(const uint8_t raw[Enc_slow::LEN_ENCODED], uint16_t light_Mean[3]);
		uint32_t light_Sum[3];
		uint16_t light_N;
		uint16_t prevLight_Mean[3];

    /*checks if data is diffrent the previous data*/
    uint8_t prevRaw[9];
};

#endif // DATASTORE_H