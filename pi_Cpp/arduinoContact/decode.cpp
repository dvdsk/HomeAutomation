#include "decode.h"

Decode::Decode(PirData* pirData_, SlowData* slowData_, 
			     SensorState* sensorState_, SignalState* signalState_){
		bufferStatus = 0;
		memset(writeBufS, 0, EncSlowFile::LEN_ENCODED);
		memset(writeBufF, 0, EncFastFile::LEN_ENCODED);

		pirData = pirData_;
		slowData = slowData_;
    sensorState = sensorState_;
		signalState = signalState_;
}

void Decode::append_Slow(const uint32_t now, const uint8_t sBuf[], 
     const uint8_t start, const uint8_t len, const uint8_t completionPart)
{
	*(writeBufS+start) |= sBuf[0]; //first byte overlaps with prev message
	memcpy(writeBufS+start+1, sBuf+1, len-1);

	std::cout<<"co2_0: "<<decode(writeBufS, EncSlowArduino::CO2, EncSlowArduino::LEN_CO2)<<"\n";

	bufferStatus |= completionPart;
	if(bufferStatus == ALL_COMPLETE){
		slowData->process(writeBufS, now);
		memset(writeBufS, 0, EncSlowFile::LEN_ENCODED);
		bufferStatus = 0;
	}	
}

void Decode::process_Slow_BED(const uint32_t now, const uint8_t sBuf[])
{
	//immidiatly decode data for state sys.
	sensorState->tempValues[temp::BED] 
	= decode(sBuf, EncSlowArduino::TEMP_BED, EncSlowArduino::LEN_TEMP);	
	sensorState->humidityValues[hum::BED] 
	= decode(sBuf, EncSlowArduino::HUM_BED, EncSlowArduino::LEN_HUM);
	sensorState->CO2ppm   
	= decode(sBuf, EncSlowArduino::CO2, EncSlowArduino::LEN_CO2);
	sensorState->Pressure 
	= decode(sBuf, EncSlowArduino::PRESSURE, EncSlowArduino::LEN_PRESSURE);
	signalState->runUpdate();

	std::cout<<sensorState->CO2ppm<<", "<<sensorState->tempValues[temp::BED] 
	         <<", "<<sensorState->humidityValues[hum::BED]<<", "
	         <<sensorState->Pressure<<"\n";

	append_Slow(now, sBuf, NODE_BED::start, 
	            NODE_BED::LEN_sBuf, NODE_BED::complete);	
}

void Decode::process_Slow_KITCHEN(const uint32_t now, const uint8_t sBuf[])
{
	//immidiatly decode data for state sys.
	sensorState->tempValues[temp::DOOR] 
	= decode(sBuf, EncSlowArduino::TEMP_DOOR, EncSlowArduino::LEN_TEMP);
	sensorState->tempValues_updated = true;
	sensorState->humidityValues[hum::DOOR] 
	= decode(sBuf, EncSlowArduino::HUM_DOOR, EncSlowArduino::LEN_HUM);
	sensorState->humidityValues_updated = true;
	signalState->runUpdate();

	append_Slow(now, sBuf, NODE_KITCHEN::start, 
	            NODE_KITCHEN::LEN_sBuf, NODE_KITCHEN::complete);
}



void Decode::process_Fast_BED(const uint32_t now, const uint8_t fBuf[])
{
	uint8_t active;
	for (int i = EncFastArduino::PIRS_BED; i<EncFastArduino::LEN_PIRS_BED; i++){
		active = (fBuf[0] & (1<<i));
		sensorState->movement[i] = !active*sensorState->movement[i] + active*now;
	}
	writeBufF[0] |= (fBuf[0]>>EncFastArduino::PIRS_BED)
	                <<EncFastFile::PIRS_BED;


	sensorState->lightValues[lght::BED] = decode(fBuf, EncFastArduino::LIGHT_BED,
	  EncFastArduino::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();
	
	slowData->preProcess_light(sensorState->lightValues, lght::BED, now);
}

void Decode::process_Fast_KITCHEN(const uint32_t now, const uint8_t fBuf[])
{
	uint8_t active;
	for (int i = EncFastArduino::PIRS_KICHEN; i<EncFastArduino::LEN_PIRS_KICHEN; i++){
		active = (fBuf[0] & (1<<i));
		sensorState->movement[i] = !active*sensorState->movement[i] + active*now;
	}
	writeBufF[0] |= (fBuf[0]>>EncFastArduino::PIRS_KICHEN)
	                <<EncFastFile::PIRS_KICHEN;

	sensorState->lightValues[lght::KITCHEN] 
	= decode(fBuf, EncFastArduino::LIGHT_BED, EncFastArduino::LEN_LIGHT);
	sensorState->lightValues[lght::DOOR] 
	= decode(fBuf, EncFastArduino::LIGHT_DOOR, EncFastArduino::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();
	
	slowData->preProcess_light(sensorState->lightValues, lght::KITCHEN, now);
	slowData->preProcess_light(sensorState->lightValues, lght::DOOR, now);
}
