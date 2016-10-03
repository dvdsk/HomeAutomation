#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time

#include <sys/stat.h> //mkdir
#include <sys/time.h>


class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
    void envirmental_write(unsigned char data[18]);
    void pir_process(unsigned char data[2]);
    

    FILE* sensDatFile;
    FILE* pirDatFile;
  private:
    unsigned char compressPir(unsigned char data);
    unsigned char prevPirData[2];    
    unsigned char pirRecord[2];
    
    const int PIR_DT = 1000; //number of milliseconds to bin the pir data to
    long long t_begin;
    struct timeval tp;


  
    int prev_halfDay;
    const int HALFDAYSEC = 86400;
    bool TimeStampSet_first;
    bool TimeStampSet_second;

    long long GetMilliSec();
    long int unix_timestamp();  
  
    bool pir_isNotSame(unsigned char data[2]);
    
    void pir_convertNotation(unsigned char B[2]);
    void pir_combine(unsigned char B[2]);
    void pir_binData(unsigned char data[2]);

    void pir_write(unsigned char data[2]);  
    void pir_writeTimestamp(long int timestamp);
};

#endif // DATASTORE_H
