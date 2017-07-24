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
uint32_t Tstamp_org = TSTAMP_ORG;
uint32_t Tstamp = Tstamp_org+RANGE;

int main(int argc, char* argv[])
{
  int startSearch;
  int stopSearch;

	//PirData* pirDat = new PirData("pirs", cache1, CACHESIZE_pir);
	SlowData* slowDat = new SlowData("slowData", cache2, CACHESIZE_slowData);

	//file1 = pirDat->getFileP();
  file2 = slowDat->getFileP();

	slowDat->findFullTS(Tstamp_org, startSearch, stopSearch);
	std::cout<<"startSearch: "<<startSearch<<"\n";
	std::cout<<"stopSearch: "<<stopSearch<<"\n";

	slowDat->findFullTS(Tstamp, startSearch, stopSearch);
	std::cout<<"startSearch: "<<startSearch<<"\n";
	std::cout<<"stopSearch: "<<stopSearch<<"\n";

	std::cout<<slowDat->getCurrentLinepos()<<"\n";
	slowDat->showHeaderData(0, slowDat->getCurrentLinepos());
	slowDat->exportAllSlowData(Tstamp_org, Tstamp);
	//delete pirDat;
	delete slowDat;

  
	return 0;
}
