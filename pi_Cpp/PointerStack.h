/**
* Class name: ArrayStack, classical array stack
* @author 
* @author 
* @file arraystack.h
* @date 08-09-2016
**/

#ifndef PointerStack_h
#define PointerStack_h

//http://www.boost.org/doc/libs/1_55_0/doc/html/boost_asio/overview/serial_ports.html
#include <boost/asio.hpp> 
using namespace::boost::asio; 
using namespace std;

#define PORT "/dev/ttyUSB0"


// Base serial settings
serial_port_base::baud_rate BAUD(9600);
serial_port_base::flow_control FLOW( serial_port_base::flow_control::none );
serial_port_base::parity PARITY( serial_port_base::parity::none );
serial_port_base::stop_bits STOP( serial_port_base::stop_bits::one );


}

#endif
