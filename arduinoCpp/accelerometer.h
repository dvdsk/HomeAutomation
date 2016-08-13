#ifndef ACCELEROMETER_H
#define ACCELEROMETER_H

#include <Arduino.h> //needed for Serial.print
#include <Wire.h> //needed for wire

class Accelerometer
{
  public:
    Accelerometer(); //constructor
    void readOut();
  private:
    int MMA7455_init(void);
    int MMA7455_xyz(uint16_t *pX, uint16_t *pY, uint16_t *pZ);
    int MMA7455_read(int start, uint8_t *buffer, int size);
    int MMA7455_write(int start, const uint8_t *pData, int size);
};


//this prevents problems if someone accidently #include's your library twice.
#endif

