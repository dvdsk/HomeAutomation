#include "SlowData.h"

void SlowData::process()

bool SlowData::newData(uint8_t raw[9], uint8_t prevRaw[9]){
  for(int i : indices(raw)){
    if(raw[i] != prevRaw[i]){ return false}
  }
  return results;
}
