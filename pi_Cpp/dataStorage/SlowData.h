#ifndef SLOWDATA_H
#define SLOWDATA_H
#include <iostream>
#include <cstring> //memcopy
#include <cstdint> //uint16_t

#include "../config.h"
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
constexpr int LIGHT_LEN = 1;
const int SLOWDATA_PACKAGESIZE = SLOWDATA_SIZE+2+2; //slow data + light data + timestamp 

//data specific functions and variables, inherits AllData
class SlowData : public Data
{
  public:    
    SlowData(const std::string filePath, uint8_t* cache, const int cacheLen);
    /*take the raw data from serial with the timestamp, rewrite it, send 
      it off for reacting if something changed and store it in a file*/
    void process(const uint8_t raw[9], const uint32_t Tstamp);

    int fetchSlowData(uint32_t startT, uint32_t stopT, 
                      double x[], double y[], int sensor);
		void preProcess_light(const uint8_t raw[FASTDATA_SIZE], const uint32_t Tstamp);

  private:
    bool newData(const uint8_t raw[SLOWDATA_SIZE], uint16_t light_Mean[LIGHT_LEN]);
		uint32_t light_Sum[LIGHT_LEN];
		uint16_t light_N;
		uint16_t prevLight_Mean[LIGHT_LEN];

    /*checks if data is diffrent the previous data*/
    uint8_t prevRaw[9];
};

inline float decodeLight(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]);
inline float decodeTemperature(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]);
inline float decodeHumidity(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]);
inline float decodeCO2(int blockIdx_B, uint8_t block[MAXBLOCKSIZE]);

#endif // DATASTORE_H
