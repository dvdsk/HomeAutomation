CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

objects = mainServer.o

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lmicrohttpd

mainServer.o: mainServer.cpp
	$(CXX) $(CXXFLAGS) -lmicrohttpd -c mainServer.cpp
#g++ -L/usr/lib/x86_64-linux-gnu -lmicrohttpd -Wall mainServer.cpp
