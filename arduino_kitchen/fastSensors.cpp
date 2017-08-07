#include "fastSensors.h"

void readAndEncode(uint8_t buffer[]){
	buffer[0] |= readPIRs();

	Serial.println("reading door\n");
	encode(buffer, BH1750::readLightLevel(ADDR_H), EncFastArduino::LIGHT_DOOR, 
	       EncFastArduino::LEN_LIGHT);
	Serial.println("reading kitchen\n");
	encode(buffer, BH1750::readLightLevel(ADDR_L), EncFastArduino::LIGHT_KITCHEN, 
	       EncFastArduino::LEN_LIGHT);
	Serial.println("done\n");
}

uint8_t readPIRs(){
	uint8_t pirStat;
	//check the PIR sensor for movement as fast as possible, this happens
	//many many times a second

	//read registery of pin bank L (fast way to read state), 
	//returns byte on is high bit off is low. See this chart for which bit in the 
	//byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
	delay(1);//crashes if removed  TODO checkthis!!!
	pirStat =  PIND & PIR_SHOES_WEST >> 2; 
	pirStat |= PIND & PIR_SHOES_EAST >> 2;
	pirStat |= PIND & PIR_DOOR >> 2; 
	pirStat |= PIND & PIR_KITCHEN >> 2;

	return pirStat;  //set bedSouth value to recieved data
}

void configure_fast(){
	Wire.begin();
	BH1750::configure(BH1750_CONTINUOUS_HIGH_RES_MODE_2, ADDR_H);
	BH1750::configure(BH1750_CONTINUOUS_HIGH_RES_MODE_2, ADDR_L);
}

void BH1750::configure(const uint8_t mode, const uint8_t addr) {

  // Check, is measurment mode exist
  switch (mode) {
    case BH1750_CONTINUOUS_HIGH_RES_MODE:
    case BH1750_CONTINUOUS_HIGH_RES_MODE_2:
    case BH1750_CONTINUOUS_LOW_RES_MODE:
    case BH1750_ONE_TIME_HIGH_RES_MODE:
    case BH1750_ONE_TIME_HIGH_RES_MODE_2:
    case BH1750_ONE_TIME_LOW_RES_MODE:

      // Send mode to sensor
      Wire.beginTransmission(addr);
      Wire.write((uint8_t)mode);
      Wire.endTransmission();

      // Wait few moments for waking up
      delay(10);
      break;

    default:
      break;
  }
}


uint16_t BH1750::readLightLevel(const uint8_t addr) {

  // Measurment result will be stored here
  uint16_t level;
	float level2;

  // Read two bytes from sensor
  Wire.requestFrom((int)addr, 2);

  // Read two bytes, which are low and high parts of sensor value
  level = Wire.read();
  level <<= 8;
  level |= Wire.read();

  // Convert raw value to lux
	level2 = level/1.2;
  level /= 1.2;

	Serial.print("level2: ");
	Serial.println(level2);

  return level;
}
