#ifndef SERIAL_H
#define SERIAL_H
#include <boost/asio.hpp>
#include <boost/thread.hpp>
#include <unistd.h>
#include <sys/ioctl.h> //for arduino reset

#include "../config.h"

//for std::chrono and sleep etc
#include <chrono>
#include <thread>

class Serial
{
  public:
    Serial(const std::string& port, const unsigned int& baud_rate);
    unsigned char readHeader();
    void readMessage(unsigned char message[], unsigned char messageLen);
		void writeString(const char* s);

  private:
  	void resetArduino();
    void writeString(const std::string& s);
    std::string readLine();

    boost::asio::io_service _io;
    boost::asio::serial_port _serial;
};

#endif // SERIAL_H

