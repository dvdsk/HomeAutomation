
//the "union" construct is useful, in which you can refer to the 
//same memory space in two different ways
typedef union
{
  int number;
  uint8_t bytes[2];
} INTUNION_t;

// Specify data and clock connections
static const int term_dataPin = 24; //PB2
static const int term_clockPin = 22; //PB0

// Specify signal connections
static const int pir_signal = 49;
static const int light_signal = 0; //anolog

// Radio connections
static const int CEPIN = 7;
static const int CSPIN = 9;
static const uint8_t ADDRESSES[][4] = { "1No", "2No", "3No" }; // Radio pipe addresses 3 bytes
//is the absolute minimum

// script setup parameters
static const int readSpeed = 1; //time between reading individual chars
static const int debugSpeed = 0; //time between reading and reply-ing used for debug
static const int resetSpeed = 0; //time for the connection to reset
static const int calibrationTime = 2000; //setup wait period

static const byte REQUESTCO2[9] = {0xFF,0x01,0x86,0x00,0x00,0x00,0x00,0x00,0x79}; 

// headers for diffrent data
// light sensors, 25 - 50 three bytes long including header
// room sensors package, 101
// pir packages are always 2 bytes long and never have a header

static const unsigned char SETUP_done = 200; //light sensor under the bed

static const unsigned char LIGHTSENS_bed = 25; //light sensor under the bed
static const unsigned char LIGHTSENS_window = 26;
static const unsigned char LIGHTSENS_kitchen = 27;
static const unsigned char LIGHTSENS_door = 28;

static const unsigned char PIRDATA = 200;//
static const unsigned char ROOMSENSORS = 202; //light sensor under the bed

int buffer[3];
int bufferLen = 0;

static const unsigned char NODE1_PIR = 1;
static const unsigned char NODE1_TEMP = 't';
static const unsigned char NODE1_TEMP_RESEND = 'd';

static const unsigned char NODE2_PIR = 2;
static const unsigned char NODE2_LIGHT = 'l';
static const unsigned char NODE2_LIGHT_RESEND = 'i';


byte rqUpdate1[1] = {NODE1_PIR};//non const as we change this to respect if we recieved
//the temperature yet, when we have not yet recieved it it's 'd' otherise its 1
byte rqUpdate2[1] = {NODE2_PIR};

int lightCounter = 0;
int accCounter = 0;
int debugCounter= 0;
int accPeriod = 200;


byte PIRs[2] {0b00000000, 0b00000000}; //stores pir data, first byte stores if a sensor has detected 
//a signal (1 = yes, 0 = no) second byte stores which sensors are included in
//this readout, [bedSouth, bedNorth, bathroomWest, bathroomEast, door, krukje,
//trashcan, heater 

static const signed short int SENSORDATA_SIZE = 9;
static const signed short int sensorData_def[SENSORDATA_SIZE] = {32767,32767,32767,32767,32767,32767,0,0,0};
signed short int sensorData[SENSORDATA_SIZE] = {32767,32767,32767,32767,32767,32767,0,0,0};
//initialised as 32767 for every value stores: temp_bed, temp_bathroom, 
//humidity_bed, humidity_bathroom, co2, light_bed, light_outside, light_door, 
//light_kitchen, whenever data is send we reset to this value


void sendPirs(){
  Serial.write(PIRDATA);
  Serial.write(PIRs[0]);
  Serial.write(PIRs[1]);
} 


void sendSensorsdata(signed short int sensorData[SENSORDATA_SIZE]){
  //used to send the data to the raspberry pi 
  //when the sensorArray has been filled
  
  INTUNION_t toSend;
  
  //header that announces the data format
  Serial.write(ROOMSENSORS);
//  Serial.println("");//FIXME
  for (unsigned int i = 0; i < SENSORDATA_SIZE; i++){
  //send 16 bit integers over serial in binairy
//    Serial.println(sensorData[i]);//FIXME
    toSend.number = sensorData[i];    
    Serial.write(toSend.bytes[0]);
    Serial.write(toSend.bytes[1]);
  }
  
  //reset sensorData to default values so we can easily check if it is complete
  memcpy(sensorData, sensorData_def, SENSORDATA_SIZE);
}

void setup()
{ 
  Serial.begin(115200); //Open serial connection to report values to host

  Serial.print("setup done, starting response loop\n");

  Serial.write(SETUP_done);
}


void loop(){
  // serial read section
  while (Serial.available()){ // this will be skipped if no data present, leading to
                              // the code sitting in the delay function below
    delay(readSpeed);  //delay to allow buffer to fill 
    if (Serial.available() >0)
    {
      int c = Serial.read(); //gets one byte from serial buffer
      if (c == 99){
        break;
      }
      buffer[bufferLen] = c;
      bufferLen++;
    }
  }

  if (bufferLen >0) {
    switch(buffer[0]){
      case 48: //acii 0
        sendSensorsdata(sensorData);//requests the remote sensor values
        //and reads in the local sensors
        break;
      case 49: //acii 1
        //nothing         
        break;
      case 50: //acii 2
        accPeriod = 1;  //fast polling    
        break;
      case 51: //acii 3
        accPeriod = 200;  //slow polling             
        break;
      case 52: //acii 4               
        break;
      default:
        //TODO replace with error code
        break;
    }//switch
  }//if

  bufferLen = 0;//empty the string

  //TODO DEBUGG measure temp periodically
  if (debugCounter > 1000) {
    sendSensorsdata(sensorData);//requests the remote sensor values
    debugCounter = 0;
    }
  sendPirs();
    
  debugCounter++;
  
  delay(resetSpeed);
}
