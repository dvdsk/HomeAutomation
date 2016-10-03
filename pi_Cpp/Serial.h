#ifndef SERIAL_H
#define SERIAL_H
#include <boost/asio.hpp>
#include <boost/thread.hpp>
#include <unistd.h>

class Serial
{
  public:
    Serial(const std::string& port, const unsigned int& baud_rate);
    unsigned char readHeader();
    void readMessage(unsigned char message[], unsigned char messageLen);

  private:
    void writeString(const std::string& s);
    std::string readLine();

    boost::asio::io_service _io;
    boost::asio::serial_port _serial;

    const static unsigned char SETUP_DONE = 200;
};

#endif // SERIAL_H
