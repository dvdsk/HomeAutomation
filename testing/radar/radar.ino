/*
this program taken from arduino Example .
  modified by By Mohannad Rawashdeh
  http://www.genotronex.com
https://www.instructables.com/

  This code used to control the digital potentiometer
  MCP41100 connected to  arduino Board
  CS >>> D10
  SCLK >> D13
  DI  >>> D11
  PA0 TO VCC
  PBO TO GND
  PW0 TO led with resistor 100ohm .
*/
#include <SPI.h>
uint8_t address = 0x11;
constexpr int CS= 10;
int i=0;

constexpr int minimum = 25;
constexpr int maximum = 200;

void setup()
{
	Serial.begin(115200);
  pinMode(CS, OUTPUT);
  SPI.begin();
}

void loop()
{
  digitalPotWrite(maximum);
	Serial.println("at max");
  delay(10000);

   digitalPotWrite(minimum);
	Serial.println("at min");
  delay(10000);
}

int digitalPotWrite(int value)
{
  digitalWrite(CS, LOW);
  SPI.transfer(address);
  SPI.transfer(value);
  digitalWrite(CS, HIGH);
}
