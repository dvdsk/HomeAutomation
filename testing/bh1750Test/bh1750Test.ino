#include <Wire.h>
#include <BH1750.h>


BH1750 lightMeter(0x23);

void setup(){
  Serial.begin(9600);
  lightMeter.begin(BH1750_CONTINUOUS_HIGH_RES_MODE_2);
  Serial.println(F("BH1750 Test"));
}


void loop() {

  uint16_t lux = lightMeter.readLightLevel();
  Serial.print("Light: ");
  Serial.print(lux);
  Serial.println(" lx");
  delay(1000);
}
