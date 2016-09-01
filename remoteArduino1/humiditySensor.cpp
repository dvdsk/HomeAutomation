//FOR REMOTE ARDUINO 1 (ARDUINO NANO), using termpin's 3 en 2 (data, sck)

#include "Arduino.h"
#include "humiditySensor.h"
#include "RF24.h"


//constructor
TempHumid::TempHumid(int dataPin, int clockPin)
{
   _dataPin = dataPin;
   _clockPin = clockPin;
}

  
void TempHumid::skipCrcSHT(int _dataPin, int _clockPin)
{
  // Skip acknowledge to end trans (no CRC)
  
  //pinMode(_dataPin, OUTPUT); //PA2
  //pinMode(_clockPin, OUTPUT); //PA0
  //-> relaced with port manipulation
  DDRD = PIN_TERM_DATA | PIN_TERM_CLOCK;

  //digitalWrite(_dataPin, HIGH);
  //digitalWrite(_clockPin, HIGH);
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_DATA | PIN_TERM_CLOCK;
  PORTD = PIN_TERM_DATA;
}


void TempHumid::waitForResultSHT(int _dataPin, void (*f1)(byte), RF24& radio)
{
  unsigned int i;
  unsigned int a;
  unsigned int ack;
  byte pipeNo, gotByte;
  
  pinMode(_dataPin, INPUT);

  for(i= 0; i < 100; ++i)
  {
    delay(10);// FIXME
    //instead of using the above delay (and wasting cycles) we run readPir
    //and other functions
    if (radio.available(&pipeNo)){
      radio.read( &gotByte, 1 );
      f1(pipeNo); //readpir and send to baseArduino
    }

    ack = PIND;
    if ((ack & 0b1000) == 0){ //if xxxx xxxx AND 0000 0100 = 1 or if the 3e bit is set // FIXME: die 0b0100 heeft vast een betekenis -> dat kan in een constante! 
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
  //  pinMode(_dataPin, INPUT);
  //  pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
  DDRD = PIN_TERM_CLOCK;
  
  val = shiftIn(_dataPin, _clockPin, 8);
  val *= 256;

  // Send the required ack
  //  pinMode(_dataPin, OUTPUT);
  //-> replaced with port manipulation
  DDRD = PIN_TERM_CLOCK | PIN_TERM_DATA;

  //  digitalWrite(_dataPin, HIGH);
  //  digitalWrite(_dataPin, LOW);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_DATA;
  PORTD = NULL;

  //  digitalWrite(_clockPin, HIGH);
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation  
  PORTD = PIN_TERM_CLOCK;
  PORTD = NULL;

  // Get the least significant bits
//  pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRD = PIN_TERM_CLOCK;
  
  val |= shiftIn(_dataPin, _clockPin, 8);
  return val;
}


void TempHumid::sendCommandSHT(int _command, int _dataPin, int _clockPin)
{
/*  unsigned int ack;*/

  // Transmission Start
  //  pinMode(_dataPin, OUTPUT);
  //  pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
  DDRD = PIN_TERM_CLOCK | PIN_TERM_DATA;
  
  digitalWrite(_dataPin, HIGH);
  digitalWrite(_clockPin, HIGH);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_CLOCK | PIN_TERM_DATA;
    
  //  digitalWrite(_dataPin, LOW);
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_CLOCK;
  PORTD = NULL;
  
  //  digitalWrite(_clockPin, HIGH);
  //  digitalWrite(_dataPin, HIGH);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_CLOCK | PIN_TERM_DATA;
  
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_DATA;

  // The command (3 msb are address and must be 000, and last 5 bits are command)
  shiftOut(_dataPin, _clockPin, MSBFIRST, _command);

  //skipping data verificatin for MOAR SPEEED
  // Verify we get the correct ack
  
  //  digitalWrite(_clockPin, HIGH);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_CLOCK | PIN_TERM_DATA;
  
  //  pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRD = PIN_TERM_CLOCK;
  
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
  
//  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTD = PIN_TERM_DATA;
  
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


float TempHumid::readTemperatureRaw(void (*f1)(byte), RF24& radio)
{
  int _val;

  // Command to send to the SHT1x to request Temperature
  int _gTempCmd  = 0b00000011;

  sendCommandSHT(_gTempCmd, _dataPin, _clockPin);
  waitForResultSHT(_dataPin, f1, radio);
  _val = getData16SHT(_dataPin, _clockPin);
  skipCrcSHT(_dataPin, _clockPin);

  return (_val);
}


float TempHumid::readTemperatureC(void (*f1)(byte), RF24& radio)
{
  int _val;                // Raw value returned from sensor
  float _temperature;      // Temperature derived from raw value

  // Conversion coefficients from SHT15 datasheet
  const float D1 = -40.0;  // for 14 Bit @ 5V
  const float D2 =   0.01; // for 14 Bit DEGC

  // Fetch raw value
  _val = readTemperatureRaw(f1, radio);

  // Convert raw value to degrees Celsius
  _temperature = (_val * D2) + D1;

  return (_temperature);
}


float TempHumid::readHumidity(float _temperature, void (*f1)(byte), RF24& radio)
{
  int _val;                    // Raw humidity value returned from sensor
  double _linearHumidity;       // Humidity with linear correction applied
  float _correctedHumidity;    // Temperature-corrected humidity

  // Conversion coefficients from SHT15 datasheet
  const float C1 = -4.0;       // for 12 Bit
  const double C2 =  0.0405;    // for 12 Bit
  const double C3 = -0.0000028; // for 12 Bit
  const float T1 =  0.01;      // for 14 Bit @ 5V
  const double T2 =  0.00008;   // for 14 Bit @ 5V

  // Command to send to the SHT1x to request humidity
  int _gHumidCmd = 0b00000101;

  // Fetch the value from the sensor
  sendCommandSHT(_gHumidCmd, _dataPin, _clockPin);
  waitForResultSHT(_dataPin, f1, radio);
  _val = getData16SHT(_dataPin, _clockPin);
  skipCrcSHT(_dataPin, _clockPin);

  // Apply linear conversion to raw value
  _linearHumidity = C1 + C2 * _val + C3 * _val * _val;

  // Correct humidity value for current temperature
  _correctedHumidity = (_temperature - 25.0 ) * (T1 + T2 * _val) + _linearHumidity;

  return (_correctedHumidity);
}
