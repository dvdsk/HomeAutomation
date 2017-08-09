#ifndef DECODE_H
#define DECODE_H

#include <memory>
#include <atomic>
#include <ctime>

#include "radio.h"
#include "../config.h"
#include "../encodingScheme.h"
#include "../dataStorage/SlowData.h"
#include "../dataStorage/PirData.h"
#include "../state/mainState.h"
#include "../compression.h"

class Decode{
	public:
		Decode(PirData* pirData_, SlowData* slowData_, 
			     SensorState* sensorState_, SignalState* signalState_);

		void process_Slow_BED(const uint32_t now, const uint8_t sBuf[]);
		void process_Slow_KITCHEN(const uint32_t now, const uint8_t sBuf[]);

		void process_Fast_BED(const uint32_t now, const uint8_t fBuf[]);
		void process_Fast_KITCHEN(const uint32_t now, const uint8_t fBuf[]);

	private:
		void decodeSlowData(uint32_t Tstamp, uint8_t writeBuf[]);
		void append_Slow(const uint32_t now, const uint8_t sBuf[], 
		     const uint8_t start, const uint8_t len, const uint8_t completionPart);

		uint8_t bufferStatus;
		uint8_t writeBufS[EncSlowFile::LEN_ENCODED];
		uint8_t	writeBufF[EncFastFile::LEN_ENCODED];

		PirData* pirData;
		SlowData* slowData;
    SensorState* sensorState;
		SignalState* signalState;
};

#endif // SERIAL_H

