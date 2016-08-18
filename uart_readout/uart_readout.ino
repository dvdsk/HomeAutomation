
const byte cmd[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 
byte response[9]; 

void setup()  
{ 
  Serial.begin(9600);         //Opens the main serial port to communicate with the computer 
  Serial1.begin(9600);        //Opens the virtual serial port with a baud of 9600 
                              //connect TX from sensor to TX1 on arduino etc
} 


//check if there is a valid respons in the serial buffer, if there is
//check if the response checksum is correct. If this is all true return the 
//ppm read out by the sensor, else return -1
int readReturn(Stream& sensorSerial){
  int responseHigh;
  int responseLow;
  int ppm;
  
  while (sensorSerial.available() > 8){
    if (!sensorSerial.read() == 0XFF){//check if reply from sensor in buffer
      sensorSerial.read();
    }
    else{
      sensorSerial.readBytes(response, 8);

      responseHigh = (int) response[1];
      responseLow = (int) response[2];
      ppm = (256*responseHigh)+responseLow;
      return ppm;
    }
  }
  return -1;
}    


void loop()  
{ 
  int ppm;
  
  Serial1.write(cmd,9);
  
  delay(2000);
  
  ppm = readReturn(Serial1); 
  Serial.println("ppm: ");
  Serial.println(ppm);
}
