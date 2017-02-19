#include "Arduino.h"
#include "config.h"
#include "co2.h"

Co2::Co2(){
	Serial1.begin(9600);  //Opens the second serial port with a baud of 9600 
		                     //connect TX from MH Co2 sensor to TX1 on arduino etc
}

void Co2::rqCO2(){
	Serial1.write(REQUESTCO2,9);// request the CO2 sensor to do a reading
	return;
}


inline byte Co2::calculate_checkV(const byte data[9]){
	byte checkV;	

	checkV = (data[1]+data[2]+data[3]+data[4]+data[5]+data[6]+data[7]);
	
	checkV = 0XFF - checkV +1;

	return checkV;
}			


int Co2::readCO2(){
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
  
  while (Serial1.available() > 8){
    if (!Serial1.read() == 0XFF){//check if reply from sensor in buffer
      Serial1.read();
    }
    else{
      Serial1.readBytes(response+1, 8);
      responseHigh = (uint8_t) response[2];
      responseLow = (uint8_t) response[3];			
			ppm = ((uint16_t)responseHigh)*256+(uint16_t)responseLow;
					
			return ppm;
    }
  }
  return -1;
}


