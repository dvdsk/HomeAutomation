#ifndef DATASTORE_H
#define DATASTORE_H
#include <hdf5/serial/hdf5.h>
#include <iostream>

class DataStore
{
  public:
		DataStore();//

  private:
		const H5::H5std_string FILE_NAME( "test.h5" );
		const H5::H5std_string  DATASET_NAME( "IntArray" );
		
		const int NX = 5;                    // dataset dimensions
		const int NY = 6;
		const int RANK = 2;
};

#endif // DATASTORE_H
