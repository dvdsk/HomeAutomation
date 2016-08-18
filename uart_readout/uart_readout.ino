
const byte cmd[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 
char response[9]; 

void setup()  
{ 
  Serial.begin(9600);         //Opens the main serial port to communicate with the computer 
  Serial1.begin(9600);        //Opens the virtual serial port with a baud of 9600 
                              //connect TX from sensor to TX1 on arduino etc
} 



unsigned char bitswap (unsigned char x){
 byte result;

   asm("mov __tmp_reg__, %[in] \n\t"
     "lsl __tmp_reg__  \n\t"   /* shift out high bit to carry */
     "ror %[out] \n\t"  /* rotate carry __tmp_reg__to low bit (eventually) */
     "lsl __tmp_reg__  \n\t"   /* 2 */
     "ror %[out] \n\t"
     "lsl __tmp_reg__  \n\t"   /* 3 */
     "ror %[out] \n\t"
     "lsl __tmp_reg__  \n\t"   /* 4 */
     "ror %[out] \n\t"
     
     "lsl __tmp_reg__  \n\t"   /* 5 */
     "ror %[out] \n\t"
     "lsl __tmp_reg__  \n\t"   /* 6 */
     "ror %[out] \n\t"
     "lsl __tmp_reg__  \n\t"   /* 7 */
     "ror %[out] \n\t"
     "lsl __tmp_reg__  \n\t"   /* 8 */
     "ror %[out] \n\t"
     : [out] "=r" (result) : [in] "r" (x));
     return(result);
}


//calculate a checksum then compare the one in the packet and return True
//if the byte checks out
bool checkChecksum(char packet[])
{
  char i, checksum;
  for( i = 0; i <7;i++){
    checksum+= packet[i];
  }
//  checksum = 0xff - checksum;
  checksum = bitswap(checksum);
  checksum += 1;
  
  
  
  Serial.println("calculated");
  Serial.println(checksum, HEX);
  Serial.println("in packet");
  Serial.println(packet[7], HEX);
  
//  checksum = bitswap(checksum);
//  Serial.println("alternative");
//  Serial.println(checksum, HEX);
    
  if (checksum == packet[7]){
    return true;
  }
  else{  
    return true; //FIXME CHANGEME
  }
}

//check if there is a valid respons in the serial buffer, if there is
//check if the response checksum is correct. If this is all true return the 
//ppm read out by the sensor, else return -1
int readReturn(Stream& sensorSerial){
  while (sensorSerial.available() > 8){
    if (!sensorSerial.read() == 0XFF){//check if reply from sensor in buffer
      sensorSerial.read();
    }
    else{
      sensorSerial.readBytes(response, 8);
      
      int i;
      for( i=0; i<8; i++){
        Serial.print("byte ");
        Serial.print(i);
        Serial.print(": ");
        Serial.println(response[i], HEX);
      }
      
      
      
      if (checkChecksum(response)){
        int responseHigh = (int) response[1];
        int responseLow = (int) response[2];
        int ppm = (256*responseHigh)+responseLow;
        return ppm;
      }
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

