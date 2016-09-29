#include "StoreData.h"

DataStore::DataStore(){
	/*
	* Create a new file using H5F_ACC_TRUNC access,
	* default file creation properties, and default file
	* access properties.
	*/
	H5File file( FILE_NAME, H5F_ACC_TRUNC );

	std::cout << "works \n";
}
//
