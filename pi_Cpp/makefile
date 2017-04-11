CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic


objects = main.o decode.o Cache.o MainData.o PirData.o SlowData.o Serial.o MainHeader.o  mainServer.o mainState.o telegramBot.o httpGetPostPut.o commandline.o webGraph.o plotly.o dygraphs.o

#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: pi
debug: CXXFLAGS += -DDEBUG -g
debug: pi
test: CXXFLAGS += -DTEST
test: pi


pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lmicrohttpd -lcurl -lboost_system -lboost_thread -lpthread -lmenu -lncurses

main.o: main.cpp config.h arduinoContact/decode.h dataStorage/MainData.h dataStorage/PirData.h \
        dataStorage/SlowData.h state/mainState.h telegramBot/telegramBot.h \
				httpServer/mainServer.h arduinoContact/Serial.h commandLine/commandline.h debug.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -c main.cpp

commandline.o: commandLine/commandline.cpp commandLine/commandline.h \
							 config.h state/mainState.h
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

mainState.o: state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -pthread -c state/mainState.cpp	

mainServer.o: httpServer/mainServer.cpp httpServer/mainServer.h httpServer/pages/webGraph.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -c httpServer/mainServer.cpp config.h
telegramBot.o: telegramBot/telegramBot.cpp telegramBot/telegramBot.h \
	telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -c telegramBot/telegramBot.cpp

webGraph.o: httpServer/pages/webGraph.cpp httpServer/pages/webGraph.h \
	httpServer/pages/dygraphs.h httpServer/pages/plotly.h config.h
	$(CXX) $(CXXFLAGS) -c httpServer/pages/webGraph.cpp

httpGetPostPut.o: telegramBot/httpGetPostPut.cpp telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -c telegramBot/httpGetPostPut.cpp	

plotly.o: httpServer/pages/plotly.h httpServer/pages/plotly.cpp
	$(CXX) $(CXXFLAGS) -c httpServer/pages/plotly.cpp

dygraphs.o: httpServer/pages/dygraphs.h httpServer/pages/dygraphs.cpp
	$(CXX) $(CXXFLAGS) -c httpServer/pages/dygraphs.cpp

.PHONY : clean
clean:
	rm pi $(objects)
