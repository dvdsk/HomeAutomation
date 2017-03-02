#include "humiditySensor.h"

//rewritten all ports to new bank, thus pin 51 (PB2) [term_dataPin] and 53 (PB0)[term_clockPin] 
//to: pin 24 (PA2) and 22 (PA0)

//constructor

void TempHumid::setup(RemoteNodes* radio_, LocalSensors* local_, 
											void(*sendFastDataPtr_)(void), uint16_t* slowData_)
{
	sendFastDataPtr = sendFastDataPtr_;
	slowData = slowData_;
	radio = radio_;
	local = local_;
}
  
void TempHumid::skipCrcSHT()
{
  // Skip acknowledge to end trans (no CRC)
  
  //pinMode(_dataPin, OUTPUT); //PA2
  //pinMode(_clockPin, OUTPUT); //PA0
  //-> relaced with port manipulation
  DDRA = PIN_TERM_DATA | PIN_TERM_CLOCK;

  //digitalWrite(_dataPin, HIGH);
  //digitalWrite(_clockPin, HIGH);
  //digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_DATA | PIN_TERM_CLOCK;
  PORTA = PIN_TERM_DATA;
}

void TempHumid::waitForResultSHT()
{
  unsigned int ack;

  pinMode(pin::TERM_DATA, INPUT);

  for(int i= 0; i < 1000; ++i)
  { 
		radio->pollNodes();
		local->updateFast_Local();		
		(*sendFastDataPtr)();
    
    ack = PINA;
    if ((ack & PIN_TERM_DATA) == 0) break;
  }
  if (ack == HIGH) {
		#ifdef DEBUG
		Serial.println("ACK TIMEOUT");	
		#endif
		//if we time out unset the slowdata aquired bits.
		*(slowData+Idx::UPDATED) = 0;
  }
}

int TempHumid::getData16SHT(){
  int val;

  // Get the most significant bits
  //  pinMode(_dataPin, INPUT);
  //  pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
  DDRA = PIN_TERM_CLOCK;
  
  val = shiftIn(pin::TERM_DATA, pin::TERM_CLOCK, 8);
  val *= 256;

  // Send the required ack
  //  pinMode(_dataPin, OUTPUT);
  //-> replaced with port manipulation
  DDRA = PIN_TERM_CLOCK | PIN_TERM_DATA;

  //  digitalWrite(_dataPin, HIGH);
  //  digitalWrite(_dataPin, LOW);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_DATA;
  PORTA = 0;

  //  digitalWrite(_clockPin, HIGH);
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation  
  PORTA = PIN_TERM_CLOCK;
  PORTA = 0;

  // Get the least significant bits
//  pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRA = PIN_TERM_CLOCK;
  
  val |= shiftIn(pin::TERM_DATA, pin::TERM_CLOCK, 8);
  return val;
}

void TempHumid::sendCommandSHT(int _command){
/*  unsigned int ack;*/

  // Transmission Start
  //  pinMode(_dataPin, OUTPUT);
  //  pinMode(_clockPin, OUTPUT);
  //-> replaced with port manipulation
  DDRA = PIN_TERM_CLOCK | PIN_TERM_DATA;

  digitalWrite(pin::TERM_DATA, HIGH);  
  digitalWrite(pin::TERM_CLOCK, HIGH);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_CLOCK | PIN_TERM_DATA;
    
  //  digitalWrite(_dataPin, LOW);
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_CLOCK;
  PORTA = 0;
  
  //  digitalWrite(_clockPin, HIGH);
  //  digitalWrite(_dataPin, HIGH);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_CLOCK | PIN_TERM_DATA;
  
  //  digitalWrite(_clockPin, LOW);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_DATA;

  // The command (3 msb are address and must be 000, and last 5 bits are command)
  shiftOut(pin::TERM_DATA, pin::TERM_CLOCK, MSBFIRST, _command);

  //skipping data verificatin for MOAR SPEEED
  // Verify we get the correct ack
  
  //  digitalWrite(_clockPin, HIGH);
  //-> replaced with port manipulation
  PORTA = PIN_TERM_CLOCK | PIN_TERM_DATA;
  
  //  pinMode(_dataPin, INPUT);
  //-> replaced with port manipulation
  DDRA = PIN_TERM_CLOCK;
  
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
  PORTA = PIN_TERM_DATA;
  
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

  sendCommandSHT(_gTempCmd);
  waitForResultSHT();
  _val = getData16SHT();
  skipCrcSHT();

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

float TempHumid::readHumidity(float tempC)
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
  sendCommandSHT(_gHumidCmd);
  waitForResultSHT();
  _val = getData16SHT();
  skipCrcSHT();

  // Apply linear conversion to raw value
  _linearHumidity = C1 + C2 * _val + C3 * _val * _val;

  // Correct humidity value for current temperature
  _correctedHumidity = (tempC - 25.0 ) * (T1 + T2 * _val) + _linearHumidity;

  return (_correctedHumidity);
}

void TempHumid::getTempHumid(){
	float tempC;
	float humid;	

	//set slowdata bits to fetched, if something times out during readTemp we unset it.
	*(slowData+Idx::UPDATED) |= (1 << Idx::TEMPERATURE_BED) | (1<<Idx::HUMIDITY_BED);

	tempC = readTemperatureC();
	humid = readHumidity(tempC);

	*(slowData+Idx::TEMPERATURE_BED) = (uint16_t)(tempC*10) +100;
	*(slowData+Idx::HUMIDITY_BED) = (uint16_t)(humid*10);


}



