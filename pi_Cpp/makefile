CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14

objects = main.o Cache.o MainData.o PirData.o Serial.o Search.o MainHeader.o

pi: $(objects)
	$(CXX) $(CXXFLAGS) -o pi main.o Cache.o MainData.o PirData.o Serial.o \
	-lboost_system -lboost_thread -lpthread

main.o: main.cpp Serial.h dataStorage/MainData.h dataStorage/PirData.h
	$(CXX) $(CXXFLAGS) -c main.cpp
Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -c dataStorage/Cache.cpp
MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
            dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -c dataStorage/MainData.cpp
PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) -c dataStorage/PirData.cpp
Serial.o: Serial.cpp
	$(CXX) $(CXXFLAGS) -c Serial.cpp	
Search.o: dataStorage/Search.cpp dataStorage/Search.h
	$(CXX) $(CXXFLAGS) -c dataStorage/Search.cpp	
MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -c dataStorage/MainHeader.cpp	

.PHONY : clean
clean:
	rm pi $(objects)
