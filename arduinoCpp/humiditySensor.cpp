#include "Arduino.h"
#include "humiditySensor.h"


//constructor
TempHumid::TempHumid(int dataPin, int clockPin)
{
   _dataPin = dataPin;
   _clockPin = clockPin;
   
   // gives power to the termp sensor
   //pinMode(term_vcc, OUTPUT);
   //-> replaced with port manipulation   
   DDRB = B00001010; 

   //digitalWrite(term_vcc, HIGH);   
   //-> replaced with port manipulation
   PORTB = B00001010; 
}

//function to fill delay with usefull shit
void TempHumid::readPIR(){
  //read registery of pin bank L (fast way to read state), 
  //returns byte on is high bit off is low. See this chart for which bit in the 
  //byte corrosponds to which pin http://forum.arduino.cc/index.php?topic=45329.0
  delay(1);//crashes if removed  
  if ((PINL & 1) != 0){
    Serial.print("m");
    }
  Serial.print("\n");
  }
  
void TempHumid::skipCrcSHT(int _dataPin, int _clockPin)
{
  // Skip acknowledge to end trans (no CRC)
  
  //pinMode(_dataPin, OUTPUT); B2
  //pinMode(_clockPin, OUTPUT); B0
  //-> relaced with port manipulation
  DDRB = B00001111; //second LSB bit too since we want the vcc of the light sensor
                    //on

  //digitalWrite(_dataPin, HIGH);
  //digitalWrite(_clockPin, HIGH);
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTB = B00001111;
  PORTB = B00001110;
}

void TempHumid::waitForResultSHT(int _dataPin)
{
  unsigned int i;
  unsigned int a;
  unsigned int ack;

  pinMode(_dataPin, INPUT);

  for(i= 0; i < 100; ++i)
  {
    //delay(10);
    //instead of using the above delay (and wasting cycles) we run readPir
    //as readpir takes about 2 milliseconds we do
    for (a=1; a < 10; ++a){
      readPIR();
    }

    ack = PINB;
    if ((ack & 4) == 0){ //if xxxx xxxx AND 0000 0100 = 1 or if the 3e bit is set
      break;
    }
  }

//  if (ack == HIGH) {
//    //Serial.println("Ack Error 2"); // Can't do serial stuff here, 
//    //need another way of reporting errors
//  }
}

int TempHumid::getData16SHT(int _dataPin, int _clockPin)
{
  int val;

  // Get the most significant bits
  //pinMode(_dataPin, INPUT);
  //pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
    
  DDRB = B00001011;
  
  val = shiftIn(_dataPin, _clockPin, 8);
  val *= 256;

  // Send the required ack
  //pinMode(_dataPin, OUTPUT);
  //-> replaced with port manipulation
  DDRB = B00001111;

  //digitalWrite(_dataPin, HIGH);
  //digitalWrite(_dataPin, LOW);
  //-> replaced with port manipulation
  PORTB = B00001110;
  PORTB = B00001010;

  //digitalWrite(_clockPin, HIGH);
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation  
  PORTB = B00001011;
  PORTB = B00001010;

  // Get the least significant bits
  //pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRB = B00001011;
  
  val |= shiftIn(_dataPin, _clockPin, 8);
  return val;
}

void TempHumid::sendCommandSHT(int _command, int _dataPin, int _clockPin)
{
/*  unsigned int ack;*/

  // Transmission Start
  //pinMode(_dataPin, OUTPUT);
  //pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
  DDRB = B00001111;  
  
  //digitalWrite(_dataPin, HIGH);
  //digitalWrite(_clockPin, HIGH);
  //-> replaced with port manipulation
  PORTB = B00001111;
    
  //digitalWrite(_dataPin, LOW);
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTB = B00001011;
  PORTB = B00001010;
  
  //digitalWrite(_clockPin, HIGH);
  //digitalWrite(_dataPin, HIGH);
  //-> replaced with port manipulation
  PORTB = B00001111;
  
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTB = B00001110;

  // The command (3 msb are address and must be 000, and last 5 bits are command)
  shiftOut(_dataPin, _clockPin, MSBFIRST, _command);

  //skipping data verificatin for MOAR SPEEED
  // Verify we get the correct ack
  
  //digitalWrite(_clockPin, HIGH);
  //-> replaced with port manipulation
  PORTB = B00001111;  
  
  //pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRB = B00001011; 
  
  //ack = digitalRead(_dataPin);
  //if (ack != LOW) {
  //  Serial.println("Ack Error 0");
  //}
  //-> replaced with port manipulation
  
  //skipping for speed
/*  ack = PINB;*/
/*  if ((ack & 4) != 0){ //if xxxx xxxx AND 0000 0100 = 1 or if the 3e bit is set*/
/*      Serial.println("Ack Error 0");*/
/*  }*/
  
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTB = B0001110; 
  
  //ack = digitalRead(_dataPin);
  //if (ack != HIGH) {
  //  Serial.println("Ack Error 1");
  //}
  //-> replaced with port manipulation

  //skipping for speed  
/*  ack = PINB;*/
/*  if ((ack & 4) != 0){ //if xxxx xxxx AND 0000 0100 = 1 or if the 3e bit is set*/
/*      Serial.println("Ack Error 1");*/
/*  }*/
}

float TempHumid::readTemperatureRaw()
{
  int _val;

  // Command to send to the SHT1x to request Temperature
  int _gTempCmd  = 0b00000011;

  sendCommandSHT(_gTempCmd, _dataPin, _clockPin);
  waitForResultSHT(_dataPin);
  _val = getData16SHT(_dataPin, _clockPin);
  skipCrcSHT(_dataPin, _clockPin);

  return (_val);
}

float TempHumid::readTemperatureC()
{
  int _val;                // Raw value returned from sensor
  float _temperature;      // Temperature derived from raw value

  // Conversion coefficients from SHT15 datasheet
  const float D1 = -40.0;  // for 14 Bit @ 5V
  const float D2 =   0.01; // for 14 Bit DEGC

  // Fetch raw value
  _val = readTemperatureRaw();

  // Convert raw value to degrees Celsius
  _temperature = (_val * D2) + D1;

  return (_temperature);
}

float TempHumid::readHumidity()
{
  int _val;                    // Raw humidity value returned from sensor
  float _linearHumidity;       // Humidity with linear correction applied
  float _correctedHumidity;    // Temperature-corrected humidity
  float _temperature;          // Raw temperature value

  // Conversion coefficients from SHT15 datasheet
  const float C1 = -4.0;       // for 12 Bit
  const float C2 =  0.0405;    // for 12 Bit
  const float C3 = -0.0000028; // for 12 Bit
  const float T1 =  0.01;      // for 14 Bit @ 5V
  const float T2 =  0.00008;   // for 14 Bit @ 5V

  // Command to send to the SHT1x to request humidity
  int _gHumidCmd = 0b00000101;

  // Fetch the value from the sensor
  sendCommandSHT(_gHumidCmd, _dataPin, _clockPin);
  waitForResultSHT(_dataPin);
  _val = getData16SHT(_dataPin, _clockPin);
  skipCrcSHT(_dataPin, _clockPin);

  // Apply linear conversion to raw value
  _linearHumidity = C1 + C2 * _val + C3 * _val * _val;

  // Get current temperature for humidity correction
  _temperature = readTemperatureC();

  // Correct humidity value for current temperature
  _correctedHumidity = (_temperature - 25.0 ) * (T1 + T2 * _val) + _linearHumidity;

  return (_correctedHumidity);
}
