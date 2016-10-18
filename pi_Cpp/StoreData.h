#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time
#include <climits> //int max etc
#include <cstdint> //uint16_t

#include <sys/stat.h> //mkdir and filesize
#include <sys/time.h>

/*
Pir saving format, normal packages with sometimes a timestamp package in front

NORMAL PIR PACKAGE:
total length 4 bytes, time short contains the lower part of the 4 byte unix time
  ----------------------------------------------------------------------------
  - time low 16 bit | pir confirmed ones 8 bit | pir confirmed zeros 8 bit -
  ----------------------------------------------------------------------------

TIMESTAMP PIR PACKAGE:
total length 4 bytes, used to store the full unixtime just in front of a normal 
pir package that crosses a treshold for putting in the full time again 
  --------------------------------------
  - time low 16 bit | time high 16 bit -
  --------------------------------------
recognised by 2 time lows after eachother


=> test if timestamp package:
this is what would be read, so test 
  -----------------
  - n | m | x | x - data block a
  ----------------- 

  -----------------
  - c | d | a | b - data block b0
  -----------------
    0   1   2   3
  -----------------
  - a | b | x | x - data block b1
  -----------------   

 a | b0 | b1 | c | d

*/

#define HALFDAYSEC 43200 //numb of sec in half a day
#define PIR_DT 1000 //number of milliseconds to bin the pir data to
#define KB 1000 //TODO replace with const shit (snap const shit eerst)

//keeps track of data and cache
class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
	  //appends one data line to the cache and writes to file
	  void write_pir(unsigned char data[4]);
	  void write_atmospheric(unsigned char data[18]);
	  void write_plants(unsigned char data[]);
	  
	  //reads one line from cache or if its not in there from the file
	  //line is the line number of the file
	  void read_pir(unsigned char data[4], int line);
	  void read_atmospheric(unsigned char data[18], int line);
	  void read_plants(unsigned char data[], int line);

    FILE* pirs_file;    
    FILE* atmospherics_file;
    FILE* plants_file;
  private:
    const static int CACHESIZE_pir = 4000;
    const static int CACHESIZE_atmospheric = 4000;    
    const static int CACHESIZE_plants = 4000;    

    const static std::string FILEPATH_pirs = "data/pirs.binDat"    
    const static std::string FILEPATH_atmospheric = "data/atmospheric.binDat" 
    const static std::string FILEPATH_plants = "data/plants.binDat"
    
    int oldest_pir; //indicates oldest element in cache
    int oldest_atmospheric;
    int oldest_plants;
    
    unsigned char cache_pir[CACHESIZE_pir];
    unsigned char cache_atmospheric[CACHESIZE_atmospheric];
    unsigned char cache_plants[CACHESIZE_plants];

    uint32_t cache_firstTime_pir;
    uint32_t cache_firstTime_atmospheric;
    uint32_t cache_firstTime_plants;

    int loadbuffer(unsigned char cache[], FILE* fileToCache, int cacheSize, 
                   uint32_t& firstTime_inCache, unsigned char packageLenght, std::string filePath);
    uint32_t TimeInFrontOfCache(FILE* fileToCache, int cacheSize, unsigned char packageLenght,
                   uint32_t firstTime_inCache);
    bool notTimePackage(unsigned char susp_time[2],  unsigned char susp_data[2]);
};

class sensorData
{
  public:
      sensorData(const int CACHESIZE, ){
        unsigned char cache[CACHESIZE];         
      }
      
      FILE* filePointer;  

      std::string FILEPATH = "data/pirs.binDat"         
      int oldestInBuffer //indicates oldest element in cache      
      uint32_t cache_firstTime;
}

//processes the data and converts to the format for storing
class PirData
{
  public:    
    PirData(StoreData& dataStorage);
    uint32_t getClosestTimeStamp(int lineNumber);
    void process(unsigned char data[2]);

  private:
    StoreData dataStorage;
    
    struct timeval tp;//TODO cant this be in the function?  

    unsigned char prevData[2];    
    unsigned char Record[2];

    long int t_begin;

    bool TimeStampSet_first;
    bool TimeStampSet_second;    


    long int unix_timestamp();  

    unsigned char compress(unsigned char data);

    bool isTimeStampPackage(unsigned char susp_time[4],  unsigned char susp_data[4]);

    bool isNotSame(unsigned char data[2]);

    void convertNotation(unsigned char B[2]);
    void combine(unsigned char B[2]);
    void binData(unsigned char data[2]);

    void putData(unsigned char data[2]);  
    void putTimestamp(long int timestamp);        
};

#endif // DATASTORE_H
