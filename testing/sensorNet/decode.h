#ifndef DECODE_H
#define DECODE_H

#include <memory>
#include <atomic>
#include <ctime>

#include "../config.h"
#include "../encodingScheme.h"
#include "../dataStorage/SlowData.h"
#include "../dataStorage/PirData.h"
#include "../state/mainState.h"
#include "../compression.h"

#include "Serial.h"
#include "encodingScheme.h"

constexpr int START_BED_NODE = TEMP_BED;
constexpr int START_KITCHEN_NODE = TEMP_BATHROOM;
constexpr int START_DOOR_NODE = TEMP_DOOR;

constexpr uint8_t COMPLETE_BED_NODE 		= 0b00000001;
constexpr uint8_t COMPLETE_KITCHEN_NODE = 0b00000010;
constexpr uint8_t COMPLETE_DOOR_NODE 		= 0b00000100;
constexpr uint8_t ALL_COMPLETE = COMPLETE_BED_NODE | 
                                 COMPLETE_KITCHEN_NODE | 
                                 COMPLETE_BATHROOM_NODE;

class Decode{
	public:
		Decode(PirData* pirData, SlowData* slowData, 
			     SensorState* sensorState, SignalState* signalState)
		process_Slow(uint32_t now);
		process_Fast();

	private:
		uint8_t bufferStatus;
		uint8_t writeBufS[EncSlowFile::LEN_ENCODED];
		uint8_t	writeBufF[EncFastFile::LEN_ENCODED];


}

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										 PirData* pirData, 
										 SlowData* slowData, 
										 SensorState* sensorState,
	                   SignalState* signalState);

void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										PirData* pirData, 
										SlowData* slowData, 
										SensorState* sensorState,
	                  SignalState* signalState);


#endif // SERIAL_H

