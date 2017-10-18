#ifndef RADIO_H
#define RADIO_H

#include <cstdlib>

#include "../config.h"
#include "../encodingScheme.h"

constexpr uint8_t COMPLETE_BED_NODE 		 = 0b00000001;
constexpr uint8_t COMPLETE_KITCHEN_NODE  = 0b00000010;
constexpr uint8_t COMPLETE_BATHROOM_NODE = 0b00000100;

constexpr uint8_t ALL_COMPLETE = COMPLETE_BATHROOM_NODE;
/*constexpr uint8_t ALL_COMPLETE = COMPLETE_BED_NODE | */
/*                                 COMPLETE_KITCHEN_NODE | */
/*                                 COMPLETE_BATHROOM_NODE;*/


namespace NODE_BED{
	constexpr uint8_t addr[] = "2Node"; //addr may only diff in first byte
	constexpr int LEN_fBuf = EncFastArduino::LEN_BED_NODE;
	constexpr int LEN_sBuf = EncSlowArduino::LEN_BED_NODE;

	constexpr int start  	 = EncSlowFile::START_BEDNODE;
	constexpr int complete = COMPLETE_BED_NODE;
}

namespace NODE_KITCHEN{
	constexpr uint8_t addr[] = "3Node"; //addr may only diff in first byte
	constexpr int LEN_fBuf = EncFastArduino::LEN_KITCHEN_NODE;
	constexpr int LEN_sBuf = EncSlowArduino::LEN_KITCHEN_NODE;

	constexpr int start  	 = EncSlowFile::START_KITCHEN;
	constexpr int complete = COMPLETE_KITCHEN_NODE;
}

namespace NODE_BATHROOM{
	constexpr uint8_t addr[] = "4Node"; //addr may only diff in first byte
	constexpr int LEN_fBuf = EncFastArduino::LEN_BATHROOM_NODE;
	constexpr int LEN_sBuf = EncSlowArduino::LEN_BATHROOMNODE;

	constexpr int start  	 = EncSlowFile::START_BATHROOM;
	constexpr int complete = COMPLETE_BATHROOM_NODE;
}

//time in which node must reply through awk package.
constexpr uint32_t MAXDURATION = 20*1000;// 20 = milliseconds

namespace pin{
	constexpr int RADIO_CE = 22;
	constexpr int RADIO_CS = 0;
}

namespace status{
	constexpr uint8_t SLOW_RDY = 0b00000001;
}

namespace headers{
	constexpr uint8_t RQ_FAST = 0;
	constexpr uint8_t RQ_MEASURE_SLOW = 1;
	constexpr uint8_t RQ_READ_SLOW = 2;
	constexpr uint8_t RQ_INIT = 3;
}

constexpr uint8_t PIPE = 1;

namespace NODE_CENTRAL{
	constexpr uint8_t addr[] = "1Node"; //addr may only diff in first byte
}

#endif // SERIAL_H

