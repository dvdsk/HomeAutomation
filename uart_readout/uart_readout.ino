
/*const byte checkValue = 0x01+0x86+00+00+00+00+00+1;*/
/*const byte requestData[] = {0XFF, 0x01, 0x86, 00, 00, 00, 00, 00, checkValue};*/

/*byte recievedData[8];*/
/*int nbuff;*/

/*void setup()                    */
/*{*/
/*  Serial.begin(9600);           // serial to computer*/
/*  Serial1.begin(9600);          // serial to Co2 meter*/
/*}*/



/*void loop()                       */
/*{*/
/*  Serial1.write(requestData, sizeof(requestData));*/
/*  nbuff = Serial1.readBytes(recievedData, 8);*/
/*  Serial.print(nbuff);*/
/*  Serial.print(checkValue);*/
/*  */
/*  for(int i = 0; i < (sizeof(recievedData) / sizeof(recievedData[0])); i++)*/
/*    Serial.print(recievedData[i]);*/

/*  */
/*  delay(2000);*/
/*                            */
/*}*/



byte checkValue = 0x01+0x86+00+00+00+00+00+1;
byte readCO2[] = {0XFF, 0x01, 0x86, 00, 00, 00, 00, 00, checkValue};
byte response[] = {0,0,0,0,0,0,0,0,0};

void setup()  
{ 
  Serial.begin(9600);         //Opens the main serial port to communicate with the computer 
  Serial1.begin(9600);    //Opens the virtual serial port with a baud of 9600 
} 
void loop()  
{ 
  sendRequest(readCO2); 
  unsigned long valCO2 = getValue(response); 
/*  */
/*  Serial.println("CO2FULLDATA_ARRAY:");*/
/*  for(int i = 0; i < (sizeof(response) / sizeof(response[0])); i++)*/
/*    Serial.print(response[i]);*/
  
/*  Serial.print("Co2 ppm = "); */
/*  Serial.println(valCO2); */
  delay(2000); 
} 


void sendRequest(byte packet[]) 
{ 
  while(!Serial1.available())  //keep sending request until we start to get a response 
  { 
    Serial1.write(readCO2,9); 
    delay(50); 
  } 
  int timeout=0;  //set a timeoute counter 
  while(Serial1.available() < 9 )  //Wait to get a 9 byte response 
  { 
    timeout++;   
    if(timeout > 10)    //if it takes to long there was probably an error 
      { 
        while(Serial1.available())  //flush whatever we have 
          Serial1.read(); 
          break;                        //exit and try again 
      } 
      delay(50); 
  } 
  for (int i=0; i < 9; i++) 
  { 
    response[i] = Serial1.read(); 
  }   
}

unsigned long getValue(byte packet[]) 
{ 
    int high = packet[4];                        //high byte for value is 4th byte in packet in the pcket 
    int low = packet[3];                         //low byte for value is 5th byte in the packet 
    
    int lowest = packet[2];                         //low byte for value is 5th byte in the packet 
    
    int extra = packet[1];                         //low byte for value is 5th byte in the packet         
   
    Serial.println(high);
    Serial.println(low);
    Serial.println(lowest);
    Serial.println(extra);
    Serial.println("done");

    unsigned long val = high*256 + low;            
    //Combine high byte and low byte with this formula to get value 
    return val; 
} 
