#include <stdio.h>

static const byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 
char buffer[3];
int bufferLen;


inline byte calculate_checkV(const byte data[9]){
	byte checkV;	

	checkV = (data[1]+data[2]+data[3]+data[4]+data[5]+data[6]+data[7]);
	
	checkV = 0XFF - checkV +1;

	return checkV;
}			


int readCO2(Stream& sensorSerial){
  //reads awnser from Co2 sensor that resides in the hardware serial buffer
  //this can be called some time after reqeusting the data thus it is not 
  //needed to wait for a reply after the request, Will also not 
  //block if no reply present but return -1 instead

  char response[9]; //changed was char, byte is better prevents issue with char
                    //in reading out the array in hex (like values such as 
                    //FFFFFFF86 instead of 86) //again to char since makefile
                    //wants it to be char really badly
  uint8_t responseHigh;
  uint8_t responseLow;
  int ppm;
  
  while (sensorSerial.available() > 8){
    if (!sensorSerial.read() == 0XFF){//check if reply from sensor in buffer
      sensorSerial.read();
    }
    else{
      sensorSerial.readBytes(response+1, 8);
      responseHigh = (uint8_t) response[2];
      responseLow = (uint8_t) response[3];			
			ppm = ((uint16_t)responseHigh)*256+(uint16_t)responseLow;
			Serial.print("\nCo2:");
			Serial.print(ppm);			
			Serial.print(" (ppm)\t\ttemp:");
			Serial.print((uint8_t)response[4]-40);
			Serial.print(" (celcius)"); 
  
			Serial.print("\trecieved check value: ");
			Serial.print((uint8_t)response[8]); 
			Serial.print("\tcalculated check value: ");
			Serial.print(calculate_checkV((uint8_t*)response));	
					
			return ppm;
    }
  }
  return -1;
}

inline uint8_t reverse_byte(uint8_t x)
{
  static const uint8_t table[] = {
      0x00, 0x80, 0x40, 0xc0, 0x20, 0xa0, 0x60, 0xe0,
      0x10, 0x90, 0x50, 0xd0, 0x30, 0xb0, 0x70, 0xf0,
      0x08, 0x88, 0x48, 0xc8, 0x28, 0xa8, 0x68, 0xe8,
      0x18, 0x98, 0x58, 0xd8, 0x38, 0xb8, 0x78, 0xf8,
      0x04, 0x84, 0x44, 0xc4, 0x24, 0xa4, 0x64, 0xe4,
      0x14, 0x94, 0x54, 0xd4, 0x34, 0xb4, 0x74, 0xf4,
      0x0c, 0x8c, 0x4c, 0xcc, 0x2c, 0xac, 0x6c, 0xec,
      0x1c, 0x9c, 0x5c, 0xdc, 0x3c, 0xbc, 0x7c, 0xfc,
      0x02, 0x82, 0x42, 0xc2, 0x22, 0xa2, 0x62, 0xe2,
      0x12, 0x92, 0x52, 0xd2, 0x32, 0xb2, 0x72, 0xf2,
      0x0a, 0x8a, 0x4a, 0xca, 0x2a, 0xaa, 0x6a, 0xea,
      0x1a, 0x9a, 0x5a, 0xda, 0x3a, 0xba, 0x7a, 0xfa,
      0x06, 0x86, 0x46, 0xc6, 0x26, 0xa6, 0x66, 0xe6,
      0x16, 0x96, 0x56, 0xd6, 0x36, 0xb6, 0x76, 0xf6,
      0x0e, 0x8e, 0x4e, 0xce, 0x2e, 0xae, 0x6e, 0xee,
      0x1e, 0x9e, 0x5e, 0xde, 0x3e, 0xbe, 0x7e, 0xfe,
      0x01, 0x81, 0x41, 0xc1, 0x21, 0xa1, 0x61, 0xe1,
      0x11, 0x91, 0x51, 0xd1, 0x31, 0xb1, 0x71, 0xf1,
      0x09, 0x89, 0x49, 0xc9, 0x29, 0xa9, 0x69, 0xe9,
      0x19, 0x99, 0x59, 0xd9, 0x39, 0xb9, 0x79, 0xf9,
      0x05, 0x85, 0x45, 0xc5, 0x25, 0xa5, 0x65, 0xe5,
      0x15, 0x95, 0x55, 0xd5, 0x35, 0xb5, 0x75, 0xf5,
      0x0d, 0x8d, 0x4d, 0xcd, 0x2d, 0xad, 0x6d, 0xed,
      0x1d, 0x9d, 0x5d, 0xdd, 0x3d, 0xbd, 0x7d, 0xfd,
      0x03, 0x83, 0x43, 0xc3, 0x23, 0xa3, 0x63, 0xe3,
      0x13, 0x93, 0x53, 0xd3, 0x33, 0xb3, 0x73, 0xf3,
      0x0b, 0x8b, 0x4b, 0xcb, 0x2b, 0xab, 0x6b, 0xeb,
      0x1b, 0x9b, 0x5b, 0xdb, 0x3b, 0xbb, 0x7b, 0xfb,
      0x07, 0x87, 0x47, 0xc7, 0x27, 0xa7, 0x67, 0xe7,
      0x17, 0x97, 0x57, 0xd7, 0x37, 0xb7, 0x77, 0xf7,
      0x0f, 0x8f, 0x4f, 0xcf, 0x2f, 0xaf, 0x6f, 0xef,
      0x1f, 0x9f, 0x5f, 0xdf, 0x3f, 0xbf, 0x7f, 0xff,
  };
  return table[x];
}

void calibrateCO2(Stream& sensorSerial, int calibrationValue){

	Serial.print("\ncalibration sensor to: ");
	Serial.print(calibrationValue);
	Serial.print("ppm, please wait for 5 seconds\n");

	byte highV = (byte)(uint16_t(calibrationValue) >> 8);
	byte lowV = (byte)(calibrationValue);

	byte data[9] = {0xFF, 0x88, 0x88, highV, lowV, 0, 0, 0, 0};
	byte checkV = calculate_checkV(data);
	data[8] = checkV;
	
	Serial.println(data[4]);
	Serial.println(data[3]);

/*	static const byte setZeroPoint[9] = {0xFF, 0x87, 0x87, 0, 0, 0, 0, 0, 0xf2};*/
/*	static const byte data2[9] = {0xFF, 0x87, 0x87, 0, 0, 0, 0, 0, 0};*/
/*	byte checkV2 = calculate_checkV(data2);*/
/*	Serial.print("checkvalue in package we sent is:");	*/
/*	Serial.print(checkV2);*/
/*	Serial.print("\n");*/

	sensorSerial.write(data, 9);
	delay(5000);
	Serial.print("Sensor calibration complete! Below are new readings using the "
							 "calibration\nif you want to calibrate again enter a new CO2 concentration\n\n");
}

void setup()
{ 
	bufferLen = 0;  

	Serial.begin(115200); //Open serial connection to report values to host
  Serial1.begin(9600);  //Opens the second serial port with a baud of 9600 
                       //connect TX from MH Co2 sensor to TX1 on arduino etc
  
  Serial.print("CO2 calibration programm started\nplease put the CO2 sensor "
							 "outside and wait till its temperature and CO2 readings stabilise\n"
							 "then enter the outside CO2 concentration in ppm. Without reference 400 ppm can be assumed\n\n");
}


void loop(){
  
	while (Serial.available() >0 ){
		char c = Serial.read(); //gets one byte from serial buffer
		buffer[bufferLen] = c;
		bufferLen++;
  }
  if (bufferLen >0) {
		int concentration;		
		sscanf(buffer, "%d", &concentration);
		calibrateCO2(Serial1, concentration);
		bufferLen = 0;
	}
	
	Serial1.write(REQUESTCO2,9);// request the CO2 sensor to do a reading	
	readCO2(Serial1);
	delay(1000);

}
