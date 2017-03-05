CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic
ROOTFLAGS = `root-config --cflags --glibs --libs`


objects = main.o decode.o Cache.o MainData.o PirData.o SlowData.o Serial.o MainHeader.o MainGraph.o mainServer.o mainState.o telegramBot.o httpGetPostPut.o commandline.o

#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: pi
debug: CXXFLAGS += -DDEBUG -g
debug: pi

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lmicrohttpd -lcurl -lboost_system -lboost_thread -lpthread -lmenu -lncurses

main.o: main.cpp config.h arduinoContact/decode.h dataStorage/MainData.h dataStorage/PirData.h \
        dataStorage/SlowData.h graph/MainGraph.h state/mainState.h telegramBot/telegramBot.h \
				httpServer/mainServer.h arduinoContact/Serial.h commandLine/commandline.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c main.cpp

commandline.o: commandLine/commandline.cpp commandLine/commandline.h graph/MainGraph.h config.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c commandLine/commandline.cpp

Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -c dataStorage/Cache.cpp
MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -c dataStorage/MainHeader.cpp	
MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
            dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c dataStorage/MainData.cpp

PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c dataStorage/PirData.cpp
SlowData.o: dataStorage/SlowData.cpp dataStorage/SlowData.h dataStorage/MainData.h \
						compression.h compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c dataStorage/SlowData.cpp

decode.o: arduinoContact/decode.cpp arduinoContact/decode.h arduinoContact/Serial.h \
					compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -c arduinoContact/decode.cpp		
Serial.o: arduinoContact/Serial.cpp arduinoContact/Serial.h
	$(CXX) $(CXXFLAGS) -c arduinoContact/Serial.cpp	

MainGraph.o: graph/MainGraph.cpp graph/MainGraph.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c graph/MainGraph.cpp	

mainState.o: state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -pthread -c state/mainState.cpp	

mainServer.o: httpServer/mainServer.cpp httpServer/mainServer.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -c httpServer/mainServer.cpp
telegramBot.o: telegramBot/telegramBot.cpp telegramBot/telegramBot.h \
	telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -c telegramBot/telegramBot.cpp

httpGetPostPut.o: telegramBot/httpGetPostPut.cpp telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -c telegramBot/httpGetPostPut.cpp	


.PHONY : clean
clean:
	rm pi $(objects)
