#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time
#include <climits> //int max etc
#include <cstdint> //uint16_t
#include <bitset>

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
--------------------------------------------
 
 
 
 ----HIGH LEVEL ACCESS (public) ----
    | storeData: storing data with a new time,*             -universal/all data
    | getting: data from specific time frames,              -universal/all data
    | removing specific time frames                         -universal/all data
 Locking mechanism to allow multi threaded/processed access (one lock per above action). implement write/read lock
 multiple readers or one write simultainiously
 
 *note: the full unix time is stored together with the raw data in a FIFO queue before the lock. The lock has a
        zero timeout. Thus time data is always accurate even if readout on another thread takes multiples of seconds
        //TODO is this needed? (might be, raspberry pi weak? implement last anyhow)
 
 ----LOW LEVEL ACCESS (private?) ----
 -Storing a new time:
    | process: adjust formatting                                                  -data specific
    | compress: check if it is really new data and needs to be saved              -data specific
    | package: add all data together and add the timestamp part                   -data specific
    the above are all data specific.the below are not.
    | write: writes the package to both cache and file                            -universal/all data
    
  -getting data from specific time frames
    | searchFT: searches for the location of the two full timestamps closest to the requested
    |           unix times                                                        -universal/all data
    | searchT:  searches onwards from the timestamps found in searchFT to find the lines of the exact times
    |           returns these lines                                               -universal/all data

*/

#define HALFDAYSEC 43200 //numb of sec in half a day
#define PIR_DT 1000 //number of milliseconds to bin the pir data to
#define KB 1000 //TODO replace with const shit (snap const shit eerst)

typedef std::bitset<32> package;

namespace Package {
    // Package::getTime(thePackage);
    // Get the 16 high bits of the package as uint16_t
    uint16_t getTime(package thePackage) {
      return (uint16_t) (thePackage >> 16).to_ulong();
    }
    // Set the 16 high bits of the package
    uint16_t setTime(uint16_t time, package thePackage) {
      
    }
}

//keeps track of data and cache
class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
	  //appends one data line to the cache and file
	  void write_pir(unsigned char data[4]);
	  void write_atmospheric(unsigned char data[18]);
	  void write_plants(unsigned char data[]);
	  
	  //reads one line from cache or if its not in there from the file
	  //line is the line number in the file this corrosponds to a diffrent offset for every 'type of data'
	  void read_pir(unsigned char data[4], int line);
	  void read_atmospheric(unsigned char data[18], int line);
	  void read_plants(unsigned char data[], int line);

    FILE* pirs_file;    
    FILE* atmospherics_file;
    FILE* plants_file;
  private: //make all these functions work for any type of package
    //on startup load the cache from the data file
    int loadbuffer(unsigned char cache[], FILE* fileToCache, int cacheSize, 
                   uint32_t& firstTime_inCache, unsigned char packageLenght, std::string filePath);
    
    //find the timestamp corrosponding with the oldest cache line (if that line is not a timestamp in itself)
    //this is a level of pir data specific logic on top of the transparant cache
    uint32_t TimeInFrontOfCache(FILE* fileToCache, int cacheSize, unsigned char packageLenght,
                   uint32_t firstTime_inCache);
    
    bool notTimePackage(unsigned char susp_time[2],  unsigned char susp_data[2]);
};

//keeps track of where data is located: file pointer, cacheSize, cache, filepath, oldest item and how old that item is
class Data
{
  public:
    Data();
    FILE* fileP; //pointer to file
    unsigned char* cache;
    std::string filePath;
		int bufferSize;
    int oldestInBuffer; //indicates oldest element in cache (=the next we will overwrite)
    uint32_t cache_firstTime;
  protected:
    int elementLength
    FILE*
};

//processes the data and converts to the format for storing
class PirData : public Data
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
