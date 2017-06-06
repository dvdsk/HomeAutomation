#include "decode.h"

uint32_t unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

void requestSensorData(Serial* arduino, 
	std::atomic<bool>* notShuttingdown){
	while(*notShuttingdown){
		arduino->writeString("0");
		std::this_thread::sleep_for(std::chrono::seconds(5));
		//std::this_thread::sleep_for(std::chrono::seconds(1));
	}
	std::cout<<"reqSensorDat shut down successfully\n";
	return;
}

void thread_checkSensorData(PirData* pirData, 
	  SlowData* slowData, 
	  SensorState* sensorState,
	  SignalState* signalState,
	  std::atomic<bool>* notShuttingdown){
  
  uint32_t Tstamp;
	uint8_t data[SLOWDATA_SIZE]; 
  uint8_t x; 
	Serial* arduino;

	try{
		arduino = new Serial("/dev/ttyUSB0", config::ARDUINO_BAUDRATE);
	}catch (boost::system::system_error const& e) {
		std::cout<<"\tCould not open serial connection on ttyUSB0,\n\t...trying ttyUSB1\n";

		try{
			arduino = new Serial("/dev/ttyUSB1", config::ARDUINO_BAUDRATE);
		}catch (boost::system::system_error const& e) {
			std::cout<<"\tCould not open serial connection on ttyUSB1\n";
			std::cout<<"\t!!!Abborting sensor readout!!!\n";
			return;
		}		
	}	

	//spawn thread that sends request for 'slow data'
	std::thread t4(requestSensorData, arduino, notShuttingdown);

	while (*notShuttingdown){
    x = arduino->readHeader();
    switch (x){      
      case headers::FAST_UPDATE:
				//std::cout<<"update fast\n";
				Tstamp = unix_timestamp();
				arduino->readMessage(data, Enc_fast::LEN_ENCODED);			
				decodeFastData(Tstamp, data, pirData, slowData, sensorState, signalState);           
        break;             
      case headers::SLOW_UPDATE:
				//std::cout<<"update slow\n";
				Tstamp = unix_timestamp();
				arduino->readMessage(data, Enc_slow::LEN_ENCODED);				
				decodeSlowData(Tstamp, data, pirData, slowData, sensorState, signalState);
				break;        
      default:
        //std::cout << "error no code matched, header: " << +x <<"\n";   
				break;  
    }
  }
	if(!t4.joinable()){ t4.join();}
	delete arduino;
	std::cout<<"Sensor readout shut down gracefully";
}

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										PirData* pirData, 
										SlowData* slowData, 
										SensorState* sensorState,
	                  SignalState* signalState){
	uint8_t temp;
	//process movement values
	//if the there has been movement recently the value temp will be one this indicates that
	//movement[] needs to be updated for that sensor. Instead of an if statement we use multiplication 
	//with temp, as temp is either 1 or 0.
	for (int i = 0; i<8; i++){
		temp = (data[0] & (1<<i)) & (data[2] & (1<<i));
		sensorState->movement[i] = !temp * sensorState->movement[i] + temp*Tstamp;
		temp = (data[1] & (1<<i)) & (data[3] & (1<<i));
		sensorState->movement[i+8] = !temp * sensorState->movement[i+8] + temp*Tstamp;
	}

	//process light values
	sensorState->lightValues[lght::BED] = 		decode(data, Enc_fast::LIGHT_BED, Enc_fast::LEN_LIGHT);
	sensorState->lightValues[lght::KITCHEN] = decode(data, Enc_fast::LIGHT_KITCHEN, Enc_fast::LEN_LIGHT);
	sensorState->lightValues[lght::DOOR] = 		decode(data, Enc_fast::LIGHT_DOOR, Enc_fast::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();//TODO check if values differ enough to warrent an update

	//store
	pirData->process(data, Tstamp);
	slowData->preProcess_light(sensorState->lightValues, Tstamp);//FIXME outside of guard
}


void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										PirData* pirData, 
										SlowData* slowData, 
										SensorState* sensorState,
	                  SignalState* signalState){
	
	//decode temp, humidity, co2 and store in state
	sensorState->tempValues[temp::BED] = 			decode(data, Enc_slow::TEMP_BED, Enc_slow::LEN_TEMP);
	sensorState->tempValues[temp::BATHROOM] = decode(data, Enc_slow::TEMP_BATHROOM, Enc_slow::LEN_TEMP);
	sensorState->tempValues[temp::DOOR] = 		decode(data, Enc_slow::TEMP_DOOR, Enc_slow::LEN_TEMP);
	sensorState->tempValues_updated = true;

	sensorState->humidityValues[hum::BED] =      decode(data, Enc_slow::HUM_BED, Enc_slow::LEN_HUM);
	sensorState->humidityValues[hum::BATHROOM] = decode(data, Enc_slow::HUM_BATHROOM, Enc_slow::LEN_HUM);
	sensorState->humidityValues[hum::DOOR] =     decode(data, Enc_slow::HUM_DOOR, Enc_slow::LEN_HUM);
	sensorState->humidityValues_updated = true;

	sensorState->CO2ppm = 	decode(data, Enc_slow::CO2, Enc_slow::LEN_CO2);
	sensorState->Pressure = decode(data, Enc_slow::PRESSURE, Enc_slow::LEN_PRESSURE);

//	std::cout<<"data1: "<<Enc_slow::CO2<<", "<<Enc_slow::LEN_CO2;
//	std::cout<<", data2: "<<Enc_slow::PRESSURE<<", "<<Enc_slow::LEN_PRESSURE;
//	std::cout<<", data3: "<<+SLOWDATA_SIZE;
//	std::cout<<", data4: "<<Enc_slow::LEN_ENCODED<<", ";
//	std::cout<<"Pressure: "<<state->Pressure<<"\n";

	signalState->runUpdate();

	//store
	slowData->process(data,Tstamp);
}
