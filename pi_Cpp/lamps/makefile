CXX = g++-6
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

objects = httpGetPostPut.o lamps.o main.o

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -lcurl -o pi $(objects)

main.o: main.cpp lamps.h httpGetPostPut.h 
	$(CXX) $(CXXFLAGS) -c main.cpp	

lamps.o: lamps.cpp httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -c lamps.cpp	

httpGetPostPut.o: httpGetPostPut.cpp httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -c httpGetPostPut.cpp	

.PHONY : clean
clean:
	rm pi $(objects)
