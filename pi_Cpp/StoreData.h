#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time

#include <sys/stat.h> //mkdir
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
  - time high 16 bit | time low 16 bit -
  --------------------------------------
time is in front so we can have 2 messages with the same low time part
 after eachother


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


class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
    void envirmental_write(unsigned char data[18]);
    void pir_process(unsigned char data[2]);
    
    void pir_readLine(int lineNumber);
    
    FILE* sensDatFile;
    FILE* pirDatFile;
  private:
    unsigned char compressPir(unsigned char data);
    unsigned char prevPirData[2];    
    unsigned char pirRecord[2];
    
    const int PIR_DT = 1000; //number of milliseconds to bin the pir data to
    long long t_begin;
    struct timeval tp;

    union Int_bytes {
    int      i;
    unsigned char bytes[2];
    }; 

    union Long_bytes {
    long int      l;
    unsigned char bytes[4];
    }; 
  
    int prev_halfDay;    

    const int HALFDAYSEC = 86400/2;
    //this way the header is desernible from the time field, as never the
    //time field can go larger then HALFDAYSEC
    const int TIMESTAMPHEADER = HALFDAYSEC+100;
    
    bool TimeStampSet_first;
    bool TimeStampSet_second;

    long long GetMilliSec();
    long int unix_timestamp();  
  
    bool pir_isTimeStampPackage(unsigned char susp_time[4],  unsigned char susp_data[4]);
  
    bool pir_isNotSame(unsigned char data[2]);
    
    void pir_convertNotation(unsigned char B[2]);
    void pir_combine(unsigned char B[2]);
    void pir_binData(unsigned char data[2]);

    void pir_write(unsigned char data[2]);  
    void pir_writeTimestamp(long int timestamp);
};

#endif // DATASTORE_H
