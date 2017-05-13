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

uint32_t unix_timestamp();

void thread_checkSensorData(std::shared_ptr<PirData> pirData, 
										 std::shared_ptr<SlowData> slowData, 
										 SensorState* sensorState,
	                   SignalState* signalState,
										 std::shared_ptr<std::atomic<bool>> notShuttingdown);

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										 SensorState* sensorState,
	                   SignalState* signalState);

void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										 SensorState* sensorState,
	                   SignalState* signalState);


#endif // SERIAL_H

