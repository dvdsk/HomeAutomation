#include "Serial.h"



Serial::Serial(const std::string& port, const unsigned int& baud_rate)
: _io(), _serial(_io,port){
  
  std::cout << "\tOpening serial port: " << port << "\n";
  _serial.set_option(boost::asio::serial_port_base::baud_rate(baud_rate));

  //wait till the arduino sends its done with initialising
	do{
		std::cout<<"\tResetting Arduino\n";
		resetArduino();
		std::this_thread::sleep_for(std::chrono::milliseconds(500));	
	}
	while(readHeader() != config::STARTUP_DONE);
  while(readHeader() != config::SETUP_DONE){
		std::this_thread::sleep_for(std::chrono::milliseconds(100));
	}
  std::cout << "\tSensors report startup completed\n";
}


void Serial::resetArduino(){
  int fd =  _serial.boost::asio::serial_port::native_handle();
  int data = TIOCM_DTR; //DTS (data terminal ready) pin of serial.
	//toggeling this pin causes a reset on the arduino
        
	ioctl(fd, TIOCMBIC, &data); //TIOCMBIC = set the status of modem bits.        
	std::this_thread::sleep_for(std::chrono::milliseconds(1));	
	ioctl(fd, TIOCMBIS, &data); //TIOCMBIS = clear the indicated modem bits.     
}

//Send message to Arduino
void Serial::writeString(const std::string& s) {
    boost::asio::write(_serial,boost::asio::buffer(s.c_str(),s.size()));
}

//Read from Arduino
std::string Serial::readLine() {

    bool end = false;
    std::string result;
    while (!end) {
        char c;
        boost::asio::read(_serial, boost::asio::buffer(&c,1));
        if(c == '\n')
            end = true;

        else if(c != '\r')
            result += c;
    }
    return result;
}

//Read from Arduino
unsigned char Serial::readHeader() {
  unsigned char c;
  boost::asio::read(_serial, boost::asio::buffer(&c,1));

  //std::cout << c << +c;
  return c;
}

void Serial::readMessage(unsigned char message[], unsigned char messageLen) {  
  boost::asio::read(_serial, boost::asio::buffer(message, messageLen));
}


