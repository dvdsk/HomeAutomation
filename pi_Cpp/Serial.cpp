#include "Serial.h"

Serial::Serial(const std::string& port, const unsigned int &baud_rate)
: _io(), _serial(_io,port)
{

    std::cout << "Opening serial port : " << port << std::endl;
    _serial.set_option(boost::asio::serial_port_base::baud_rate(baud_rate));
    /*
     *Warning ! Arduino resets when its serial port is opened !
     *That's why we must wait until arduino is operationnal again
     *If you do not want to wait you can :
     *Add a 10 µF capacitor between GROUND and RESET
     *But if you want to program your arduino you will have to disconnect the capacitor
    */
    //Wait 5 seconds for the arduino to be ready :
    boost::this_thread::sleep(boost::posix_time::seconds(5));
}

//Send message to Arduino
void Serial::writeString(const std::string& s)
{
    boost::asio::write(_serial,boost::asio::buffer(s.c_str(),s.size()));
}

//Read from Arduino
std::string Serial::readLine()
{

    bool end = false;
    std::string result;
    while (!end)
    {
        char c;
        boost::asio::read(_serial, boost::asio::buffer(&c,1));
        if(c == '\n')
            end = true;

        else if(c != '\r')
            result += c;
    }
    return result;
}

/*Put your public methods to communicate with your Arduino here
 *Examples (using sketch_arduino.ino) :
 */
std::string Serial::getHelloResponse()
{
    std::cout << "sending  \"hello\"" << std::endl;
    writeString("hello\n");
    return readLine();
}

std::string Serial::switchLed()
{
    std::cout << "switching led" << std::endl;
    writeString("switchled\n");
    return readLine();
}
