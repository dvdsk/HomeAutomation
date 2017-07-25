#include <iostream>
#include <ctime>
#include <signal.h>

#include "dataStorage/MainData.h"
#include "dataStorage/PirData.h"
#include "dataStorage/SlowData.h"
#include "config.h"

const std::string PATHPIR = "pirs.binDat";
const int CACHESIZE_pir      = pirData::PACKAGESIZE*2;
const int CACHESIZE_slowData = slowData::PACKAGESIZE*2;

//cache for data
uint8_t cache1[CACHESIZE_pir];
uint8_t cache2[CACHESIZE_slowData];

FILE* file1; //needed as global for interrupt handling
FILE* file2;

void interruptHandler(int s){
  //fflush(file1);
  fflush(file2);
  printf("Caught signal %d\n",s);
  exit(1); 
}

std::atomic<int> lightValues[lght::LEN];
uint8_t data[SLOWDATA_SIZE];
uint32_t Tstamp_org;
uint32_t Tstamp;
unsigned int loc1, loc2;

struct stat filestatus;
int fileSize; //in bytes

int main(int argc, char* argv[])
{
	//remove("data/slowData.binDat");
	//remove("data/slowData.header");

	//get the file size
  stat("data/slowData.binDat", &filestatus);//sys call for file info
  fileSize = filestatus.st_size;

	//PirData* pirDat = new PirData("pirs", cache1, CACHESIZE_pir);
	SlowData* slowDat = new SlowData("slowData", cache2, CACHESIZE_slowData);

	if(fileSize == 0){Tstamp_org = TSTAMP_ORG; std::cout<<"USING TSTAMP_ORG\n"; }
	else{
		slowDat->searchTstamps(0, -1, loc1, loc2);
		Tstamp_org = slowDat->getTimeAt(loc2);
	}
	Tstamp = Tstamp_org;

	//file1 = pirDat->getFileP();
  file2 = slowDat->getFileP();

	

	for(unsigned int i = 0; i<RANGE; i++){
		Tstamp += 100;
		//std::cout<<"Tstamp: "<<Tstamp<<"\t\tContinuing at: "<<Tstamp_org<<"\n";
		lightValues[lght::BED] += 1;	
		slowDat->preProcess_light(lightValues, Tstamp);
		slowDat->preProcess_light(lightValues, Tstamp);
		slowDat->process(data, Tstamp);
	}

	//delete pirDat;
	delete slowDat;

  
	return 0;
}
