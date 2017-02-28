#ifndef DECODE_H
#define DECODE_H

#include "../config.h"
#include "../dataStorage/SlowData.h"
#include "../dataStorage/PirData.h"
#include "../state/mainState.h"
#include "../compression.h"

#include "Serial.h"

uint32_t unix_timestamp();

void checkSensorData(std::shared_ptr<PirData> pirData, 
										 std::shared_ptr<SlowData> slowData, 
										 std::shared_ptr<MainState> state);

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state);

void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state);


#endif // SERIAL_H

