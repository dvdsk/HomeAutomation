#include "decode.h"

uint32_t unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

void checkSensorData(PirData* pirData, SlowData* slowData, MainState* state){
  
  const unsigned char POLLING_FAST = 200;   //PIR and light Level
  const unsigned char POLLING_SLOW = 202;   //Temperature, humidity and co2
  
  uint32_t Tstamp;
	uint8_t data[SLOWDATA_SIZE]; 
  uint8_t x; 
 
  Serial arduino("/dev/ttyUSB0",115200);
  while (true){
    x = arduino.readHeader();
    switch (x){      
      case POLLING_FAST:
				Tstamp = unix_timestamp();
				arduino.readMessage(data, FASTDATA_SIZE);			
				decodeFastData(Tstamp, data, pirData, slowData, state);           
        break;             
      case POLLING_SLOW:
				Tstamp = unix_timestamp();
				arduino.readMessage(data, SLOWDATA_SIZE);				
				decodeSlowData(Tstamp, data, pirData, slowData, state);
				break;        
      default:
        std::cout << "error no code matched, header: " << +x <<"\n";     
    }
  }
}

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE], 
PirData* pirData, SlowData* slowData, MainState* state){
	uint8_t temp;
	//process movement values
	//if the there has been movement recently the value temp will be one this indicates that
	//movement[] needs to be updated for that sensor. Instead of an if statement we use multiplication 
	//with temp, as temp is either 1 or 0.
	for (int i = 0; i<5; i++){
		temp = (data[Idx_fast::pirs] & (0b00001<<i)) & (data[Idx_fast::pirs_updated] & (0b00001<<i));
		state->movement[i] = !temp * state->movement[i] + temp*Tstamp;
	}

	//process light values
	state->lightValues[lght::BED] = decodeLight(Idx_fast::LIGHT_BED, data);
	state->lightValues_updated = true;

	//store
	pirData->process(data, Tstamp);
	slowData->preProcess_light(data, Tstamp);
}


void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE], 
PirData* pirData, SlowData* slowData, MainState* state){

	//decode temp, humidity, co2 and store in state
	state->tempValues[temp::BED] = decodeTemperature(Idx_slow::TEMP_BED, data);
	state->tempValues[temp::BATHROOM] = decodeTemperature(Idx_slow::TEMP_BATHROOM, data);
	state->tempValues[temp::DOOR] = decodeTemperature(Idx_slow::TEMP_DOOR, data);
	state->tempValues_updated = true;

	state->humidityValues[hum::BED] = decodeHumidity(Idx_slow::HUM_BED, data);
	state->humidityValues[hum::BATHROOM] = decodeHumidity(Idx_slow::HUM_BATHROOM, data);
	state->humidityValues[hum::DOOR] = decodeHumidity(Idx_slow::HUM_DOOR, data);
	state->humidityValues_updated = true;

	state->CO2ppm = decodeCO2(Idx_slow::CO2, data);
	
	//store
	slowData->process(data,Tstamp);
}
