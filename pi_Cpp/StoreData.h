#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy

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
    
    const int PIR_DT = 2000; //number of milliseconds to bin the pir data to
    long long t_begin;
    struct timeval tp;

    long long GetMilliSec();
  
    bool pir_isNotSame(unsigned char data[2]);
    
    void pir_convertNotation(unsigned char B[2]);
    void pir_combine(unsigned char B[2]);
    void pir_binData(unsigned char data[2]);

    void pir_write(unsigned char data[2]);  

};

#endif // DATASTORE_H
