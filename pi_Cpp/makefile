CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic
ROOTFLAGS = `root-config --cflags --glibs`


objects = main.o Cache.o MainData.o PirData.o Serial.o MainHeader.o MainGraph.o

#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: pi
debug: CXXFLAGS += -DDEBUG -g
debug: pi

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lboost_system -lboost_thread -lpthread

main.o: main.cpp Serial.h dataStorage/MainData.h dataStorage/PirData.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c main.cpp
Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -c dataStorage/Cache.cpp
MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -c dataStorage/MainHeader.cpp	
MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
            dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c dataStorage/MainData.cpp
PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c dataStorage/PirData.cpp
Serial.o: Serial.cpp
	$(CXX) $(CXXFLAGS) -c Serial.cpp	
MainGraph.o: graph/MainGraph.cpp graph/MainGraph.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c graph/MainGraph.cpp	

.PHONY : clean
clean:
	rm pi $(objects)
