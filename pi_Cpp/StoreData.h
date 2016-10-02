#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>

#include <sys/stat.h>

class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
    void envirmental_write(unsigned char data[18]);
    void pir_write(unsigned char data[2]);

    

    FILE* sensDatFile;
    FILE* pirDatFile;
  private:
    unsigned char compressPir(unsigned char data);
    unsigned char prevPirData[2];    
    std::chrono::time_point lastPirBegin;
    int PIR_DT = 200; //number of milliseconds to bin the pir data to

    void pir_process(unsigned char data[2]);
    
    bool pir_checkIfSame(unsigned char data[2]);
    
    void pir_convertNotation(unsigned char& B[2]);
    void pir_combine(unsigned char& B[2]);
    void pir_binData(unsigned char data[2]);

};

#endif // DATASTORE_H
