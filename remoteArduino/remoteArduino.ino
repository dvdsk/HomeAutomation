#include <SPI.h>
#include <RH_NRF24.h>

RH_NRF24 nrf24;

void setup() {
  // put your setup code here, to run once:
  Serial.begin(115200);
  if (!nrf24.init())
    Serial.println("init failed");
  // Defaults after init are 2.402 GHz (channel 2), 2Mbps, 0dBm
  if (!nrf24.setChannel(1))
    Serial.println("setChannel failed");
  if (!nrf24.setRF(RH_NRF24::DataRate2Mbps, RH_NRF24::TransmitPower0dBm))
    Serial.println("setRF failed");
   //give the pir sensor some time to calibrate
   delay(2000);  
  Serial.println("setup done");
}

void recieveRf()
{
  Serial.println("Sending to nrf24_server");
  // Send a message to nrf24_server
  uint8_t data[] = "Hello World!";
  nrf24.send(data, sizeof(data));
  
  nrf24.waitPacketSent();
  // Now wait for a reply

  uint8_t buf[RH_NRF24_MAX_MESSAGE_LEN];
  uint8_t len = sizeof(buf);

  if (nrf24.waitAvailableTimeout(500))
  { 
    // Should be a reply message for us now   
    if (nrf24.recv(buf, &len))
    {
      Serial.print("got reply: ");
      Serial.println((char*)buf);
    }
    else
    {
      Serial.println("recv failed");
    }
  }
  else
  {
    Serial.println("No reply, is nrf24_server running?");
  }
  delay(400);
}

void sendRf()
{
  if (nrf24.available())
  {
    uint8_t data[] = "boe";
    // Should be a message for us now   
    uint8_t buf[RH_NRF24_MAX_MESSAGE_LEN];
    uint8_t len = sizeof(buf);
    if (nrf24.recv(buf, &len))
    {
//      NRF24::printBuffer("request: ", buf, len);
      Serial.print("got request: ");
      Serial.println((char*)buf);
      
      int val = digitalRead(2);
      Serial.println(val);
      Serial.println(PIND);
      
      // Send a reply
      
      //read pir
      if ((PIND & 4) != 0){//checks pir signal on port 3 (aka binairy value 4)
        uint8_t data[] = "m";
        Serial.println("SUCCESSS");
      }
      else{
        uint8_t data[] = "boe";
      }  
      
      nrf24.send(data, sizeof(data));
      nrf24.waitPacketSent();
    }
    else
    {
      Serial.println("recv failed");
    }
  }
}

void loop(){
  sendRf();
  //recieveRf();
}
