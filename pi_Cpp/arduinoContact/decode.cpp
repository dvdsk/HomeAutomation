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

//	std::cout<<sensorState->CO2ppm<<", "<<sensorState->tempValues[temp::BED] 
//	         <<", "<<sensorState->humidityValues[hum::BED]<<", "
//	         <<sensorState->Pressure<<"\n";

	append_Slow(now, sBuf, roundUp(EncSlowFile::START_BEDNODE,8), 
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

	append_Slow(now, sBuf, roundUp(EncSlowFile::START_KITCHEN,8), 
	            NODE_KITCHEN::LEN_sBuf, NODE_KITCHEN::complete);
}

void Decode::process_Slow_BATHROOM(const uint32_t now, const uint8_t sBuf[])
{
	//immidiatly decode data for state sys.
	sensorState->tempValues[temp::BATHROOM] 
	= decode(sBuf, EncSlowArduino::TEMP_BATHROOM, EncSlowArduino::LEN_TEMP);
	sensorState->tempValues_updated = true;
	//std::cout<<sensorState->tempValues[temp::BATHROOM]<<"\n";


	sensorState->humidityValues[hum::BATHROOM] 
	= decode(sBuf, EncSlowArduino::HUM_BATHROOM, EncSlowArduino::LEN_HUM);
	sensorState->humidityValues_updated = true;
	std::cout<<sensorState->humidityValues[hum::BATHROOM]<<"\n";

	signalState->runUpdate();

	append_Slow(now, sBuf, roundUp(EncSlowFile::START_BATHROOM,8), 
	            NODE_BATHROOM::LEN_sBuf, NODE_BATHROOM::complete);
}

void Decode::append_Fast(const uint32_t now, const uint8_t fBuf[], 
     const uint8_t start, const uint8_t len, const uint8_t completionPart)
{
	*(writeBufF+start) |= fBuf[0]; //first byte overlaps with prev message
	memcpy(writeBufF+start+1, fBuf+1, len-1);

	bufferStatus |= completionPart;
//	if(bufferStatus == ALL_COMPLETE){//TODO fix me
//		pirData->process(writeBufF, now);
//		memset(writeBufS, 0, EncSlowFile::LEN_ENCODED);
//		bufferStatus = 0;
//	}	
}

void Decode::process_Fast_BED(const uint32_t now, const uint8_t fBuf[])
{
	
	uint8_t active;
	for (int i =EncFastArduino::PIRS_BED, j =mov::BATHROOM_WC; i<EncFastArduino::LEN_PIRS_BED; i++, j++){
		active = (fBuf[0] & (1<<i));
		sensorState->movement[j] = !active*sensorState->movement[j] + active*now;
	}
//	writeBufF[0] |= (fBuf[0]>>EncFastArduino::PIRS_BED)
//	                <<EncFastFile::PIRS_BED;

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
//	writeBufF[0] |= (fBuf[0]>>EncFastArduino::PIRS_KICHEN)
//	                <<EncFastFile::PIRS_KICHEN;

	sensorState->lightValues[lght::KITCHEN] 
	= decode(fBuf, EncFastArduino::LIGHT_BED, EncFastArduino::LEN_LIGHT);
	sensorState->lightValues[lght::DOOR] 
	= decode(fBuf, EncFastArduino::LIGHT_DOOR, EncFastArduino::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();
	
	slowData->preProcess_light(sensorState->lightValues, lght::KITCHEN, now);
	slowData->preProcess_light(sensorState->lightValues, lght::DOOR, now);
}

void Decode::process_Fast_BATHROOM(const uint32_t now, const uint8_t fBuf[])
{
	uint8_t active;
	//if(fBuf[0] != 0)
	//std::cout<<"fBuf[0]: "<<+fBuf[0]<<"\n";
	for (int i =EncFastArduino::PIRS_BATHROOM, j =mov::BATHROOM_WC; i<EncFastArduino::PIRS_BATHROOM+EncFastArduino::LEN_PIRS_BATHROOM; i++, j++){
		active = (fBuf[0] & (1<<i))>>i;
		//std::cout<<"active: "<<+active<<" "<<(1<<i)<<" "<<i<<"\n";
		sensorState->movement[j] = !active*sensorState->movement[j] + active*now;
		//TODO
		if((fBuf[0] & (1<<i))>>i)
			std::cout<<sensorState->movement[j]<<"\n";

	}
//	std::cout<<"BATHROOM_WC: "<<sensorState->movement[mov::BATHROOM_WC]
//	         <<" BATHROOM_SHOWER: "<<sensorState->movement[mov::BATHROOM_SHOWER]<<"\n";
	writeBufF[0] |= (fBuf[0]>>EncFastArduino::PIRS_BATHROOM)
	                <<EncFastFile::PIRS_BATHROOM;

	append_Fast(now, writeBufF, EncFastFile::PIRS_BATHROOM, 
	            roundUp(EncFastFile::LEN_PIRS_BATHROOM,8), NODE_BATHROOM::complete);

	signalState->runUpdate();
}
