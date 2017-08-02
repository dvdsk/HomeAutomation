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

/*constexpr int START_BED_NODE = TEMP_BED;*/
/*constexpr int START_KITCHEN_NODE = TEMP_BATHROOM;*/
/*constexpr int START_DOOR_NODE = TEMP_DOOR;*/

constexpr uint8_t COMPLETE_BED_NODE 		 = 0b00000001;
constexpr uint8_t COMPLETE_KITCHEN_NODE  = 0b00000010;
constexpr uint8_t COMPLETE_BATHROOM_NODE = 0b00000100;
constexpr uint8_t ALL_COMPLETE = COMPLETE_BED_NODE;// | 
/*                                 COMPLETE_KITCHEN_NODE | */
/*                                 COMPLETE_BATHROOM_NODE;*/

class Decode{
	public:
		Decode(PirData* pirData_, SlowData* slowData_, 
			     SensorState* sensorState_, SignalState* signalState_);
		void process_Slow(const uint32_t now, const uint8_t sBuf[], 
     		 const uint8_t start, const uint8_t len, const uint8_t completionPart);
		void process_Fast_BED(const uint32_t now, const uint8_t fBuf[]);
		void process_Fast_KITCHEN(const uint32_t now, const uint8_t fBuf[]);

	private:
		void decodeSlowData(uint32_t Tstamp, uint8_t writeBuf[]);

		uint8_t bufferStatus;
		uint8_t writeBufS[EncSlowFile::LEN_ENCODED];
		uint8_t	writeBufF[EncFastFile::LEN_ENCODED];

		PirData* pirData;
		SlowData* slowData;
    SensorState* sensorState;
		SignalState* signalState;
};

#endif // SERIAL_H

