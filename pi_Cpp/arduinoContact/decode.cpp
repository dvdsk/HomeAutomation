#include "decode.h"

uint32_t unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

void requestSensorData(std::shared_ptr<Serial> arduino){
	while(1){	
		arduino->writeString("0");
		std::cout<<"requesting new data from arduino\n";
		std::this_thread::sleep_for(std::chrono::seconds(5));
	}
}

void checkSensorData(std::shared_ptr<PirData> pirData, 
										 std::shared_ptr<SlowData> slowData, 
										 std::shared_ptr<MainState> state){
  
  uint32_t Tstamp;
	uint8_t data[SLOWDATA_SIZE]; 
  uint8_t x; 

	std::shared_ptr<Serial> arduino = std::make_shared<Serial>("/dev/ttyUSB0",
																		config::ARDUINO_BAUDRATE);
	
	//spawn thread that sends request for 'slow data'
	std::thread t4(requestSensorData, arduino);

	while (true){
    x = arduino->readHeader();
    switch (x){      
      case headers::FAST_UPDATE:
				std::cout<<"update fast\n";
				Tstamp = unix_timestamp();
				arduino->readMessage(data, Enc_fast::LEN_ENCODED);			
				decodeFastData(Tstamp, data, pirData, slowData, state);           
        break;             
      case headers::SLOW_UPDATE:
				std::cout<<"update slow\n";
				Tstamp = unix_timestamp();
				arduino->readMessage(data, Enc_slow::LEN_ENCODED);				
				decodeSlowData(Tstamp, data, pirData, slowData, state);
				break;        
      default:
        std::cout << "error no code matched, header: " << +x <<"\n";     
    }
  }
	t4.join();
}

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state){
	uint8_t temp;
	//process movement values
	//if the there has been movement recently the value temp will be one this indicates that
	//movement[] needs to be updated for that sensor. Instead of an if statement we use multiplication 
	//with temp, as temp is either 1 or 0.
	for (int i = 0; i<8; i++){
		temp = (data[0] & (1<<i)) & (data[2] & (1<<i));
		state->movement[i] = !temp * state->movement[i] + temp*Tstamp;
		temp = (data[1] & (1<<i)) & (data[3] & (1<<i));
		state->movement[i+8] = !temp * state->movement[i+8] + temp*Tstamp;
	}

	//process light values
	state->lightValues[lght::BED] = 		decode(data, Enc_fast::LIGHT_BED, Enc_fast::LEN_LIGHT);
	state->lightValues[lght::KITCHEN] = decode(data, Enc_fast::LIGHT_KITCHEN, Enc_fast::LEN_LIGHT);
	state->lightValues[lght::DOOR] = 		decode(data, Enc_fast::LIGHT_DOOR, Enc_fast::LEN_LIGHT);
	state->lightValues_updated = true;

//	std::cout<<"\t"<<state->lightValues[lght::BED]<<"\n";
//	std::cout<<"\t"<<state->lightValues[lght::KITCHEN]<<"\n";
//	std::cout<<"\t"<<state->lightValues[lght::DOOR]<<"\n";

	//store
	pirData->process(data, Tstamp);
	slowData->preProcess_light(state->lightValues, Tstamp);
}


void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state){

	//decode temp, humidity, co2 and store in state
	state->tempValues[temp::BED] = 			decode(data, Enc_slow::TEMP_BED, Enc_slow::LEN_TEMP);
	state->tempValues[temp::BATHROOM] = decode(data, Enc_slow::TEMP_BATHROOM, Enc_slow::LEN_TEMP);
	state->tempValues[temp::DOOR] = 		decode(data, Enc_slow::TEMP_DOOR, Enc_slow::LEN_TEMP);
	state->tempValues_updated = true;

	state->humidityValues[hum::BED] = decode(data, Enc_slow::HUM_BED, Enc_slow::LEN_TEMP);
	state->humidityValues[hum::BATHROOM] = decode(data, Enc_slow::HUM_BATHROOM, Enc_slow::LEN_TEMP);
	state->humidityValues[hum::DOOR] = decode(data, Enc_slow::HUM_DOOR, Enc_slow::LEN_TEMP);
	state->humidityValues_updated = true;

	state->CO2ppm = decode(data, Enc_slow::CO2, Enc_slow::LEN_CO2);
	
	std::cout<<"\t"<<state->tempValues[temp::BED]<<"\n";
	std::cout<<"\t"<<state->tempValues[temp::BATHROOM]<<"\n";
	std::cout<<"\t"<<state->tempValues[temp::DOOR]<<"\n";
	std::cout<<"\t"<<state->humidityValues[hum::BED]<<"\n";
	std::cout<<"\t"<<state->humidityValues[hum::BATHROOM]<<"\n";
	std::cout<<"\t"<<state->humidityValues[hum::DOOR]<<"\n";
	std::cout<<"\t"<<state->CO2ppm<<"\n";


	//store
	slowData->process(data,Tstamp);
}
