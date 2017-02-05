#include "config.h"
namespace config {
	constexpr int HTTPSERVER_PORT = 8443;
	constexpr char* HTTPSERVER_USER = "kleingeld";
	constexpr char* HTTPSERVER_PASS = "nRhRudGLWs35rHukzxrz"; //using random strings as passw
};

namespace lcht {//lightvalues
	constexpr int DOOR = 0
	constexpr int KITCHEN = 1
	constexpr int BED = 2
};

namespace mov {//movement sensors
	constexpr int DOOR = 0
	constexpr int KITCHEN = 1
	constexpr int BED_l = 2
	constexpr int BED_r = 3
	constexpr int RADIATOR = 4
	constexpr int MIDDLEROOM = 5
};

namespace lmps {//lamps
	constexpr int DOOR = 0
	constexpr int KITCHEN = 1
	constexpr int CEILING = 2
	constexpr int BATHROOM = 3
	constexpr int RADIATOR = 4
	constexpr int BUREAU = 5
};
#endif
