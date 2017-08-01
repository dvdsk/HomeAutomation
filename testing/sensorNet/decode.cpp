#include "decode.h"

void process_Slow(const uint32_t now, const uint8_t sBuf[], 
     const uint8_t start, const uint8_t len, const uint8_t completionPart)
{
	memcpy(writeBuf+start, sBuf, len);
	bufferStatus |= completionPart;
	if(bufferStatus == ALL_COMPLETE){
		decodeSlowData(now, writeBuf);
		bufferStatus = 0;
	}	
}

void process_Fast_BED(const uint32_t now, const uint8_t fBuf[])
{
	for (int i = PIRS_BED; i<LEN_PIRS_BED; i++){
		active = (data[0] & (1<<i));
		sensorState->movement[i] = !active*sensorState->movement[i] + active*now;
	}
	uint8_t writeBufF |= (fBuf>>EncFastArduino::PIRS_BED)
	        <<EncFastFile::PIRS_BED;


	sensorState->lightValues[lght::BED] = decode(data, Enc_fast::LIGHT_BED, Enc_fast::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();
	
	slowData->preProcess_light(sensorState->lightValues[lght::BED], now)

//	uint8_t writeBufF |= (fBuf>>EncFastArduino::PIRS_KICHEN)
//	        <<EncFastFile::PIRS_KICHEN;

}

void process_Fast_KITCHEN(const uint32_t now, const uint8_t fBuf[])
{
	for (int i = PIRS_BED; i<LEN_PIRS_BED; i++){
		active = (data[0] & (1<<i));
		sensorState->movement[i] = !active*sensorState->movement[i] + active*now;
	}
	uint8_t writeBufF |= (fBuf>>EncFastArduino::PIRS_KICHEN)
	        <<EncFastFile::PIRS_KICHEN;

	sensorState->lightValues[lght::KITCHEN] 
	= decode(data, Enc_fast::LIGHT_BED, Enc_fast::LEN_LIGHT);
	sensorState->lightValues[lght::DOOR] 
	= decode(data, Enc_fast::DOOR, Enc_fast::LEN_LIGHT);
	sensorState->lightValues_updated = true;
	signalState->runUpdate();
	
	slowData->preProcess_light(sensorState->lightValues[lght::KITCHEN], now);
	slowData->preProcess_light(sensorState->lightValues[lght::DOOR], now);
}

void decodeSlowData(uint32_t Tstamp, uint8_t writeBuf[]){
	
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

	signalState->runUpdate();

	//store
	slowData->process(writeBuf,Tstamp);
}
