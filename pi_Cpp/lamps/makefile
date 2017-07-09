CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

objects = HttpSocket.o lamps.o main.o

pi: $(objects)
	$(CXX) $(CXXFLAGS) -o pi $(objects)

main.o: main.cpp lamps.h ../smallFunct/HttpSocket.h
	$(CXX) $(CXXFLAGS) -c main.cpp	

lamps.o: lamps.cpp ../smallFunct/HttpSocket.h
	$(CXX) $(CXXFLAGS) -c lamps.cpp	

HttpSocket.o: ../smallFunct/HttpSocket.cpp ../smallFunct/HttpSocket.h
	$(CXX) $(CXXFLAGS) -c ../smallFunct/HttpSocket.cpp	

.PHONY : clean
clean:
	rm pi $(objects)
