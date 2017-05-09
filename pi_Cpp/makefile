MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

ODIR = .BUILD
OBJS = $(addprefix $(ODIR)/,main.o decode.o Cache.o MainData.o PirData.o SlowData.o Serial.o MainHeader.o  mainServer.o mainState.o telegramBot.o httpGetPostPut.o commandline.o webGraph.o mpd.o lamps.o)

#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: pi
debug: CXXFLAGS += -DDEBUG -g
debug: pi

pi: $(OBJS)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(OBJS) \
	-lmicrohttpd -lcurl -lboost_system -lboost_thread -lpthread -lmenu -lncurses

$(ODIR)/main.o: main.cpp config.h arduinoContact/decode.h \
	dataStorage/MainData.h dataStorage/PirData.h dataStorage/SlowData.h \
	state/mainState.h telegramBot/telegramBot.h \
	httpServer/mainServer.h arduinoContact/Serial.h commandLine/commandline.h debug.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o$@ -c main.cpp

$(ODIR)/commandline.o: commandLine/commandline.cpp commandLine/commandline.h \
	config.h state/mainState.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o$@ -c commandLine/commandline.cpp

$(ODIR)/Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/Cache.cpp
$(ODIR)/MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainHeader.cpp	
$(ODIR)/MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
  dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o$@ -c dataStorage/MainData.cpp

$(ODIR)/PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o$@ -c dataStorage/PirData.cpp
$(ODIR)/SlowData.o: dataStorage/SlowData.cpp dataStorage/SlowData.h dataStorage/MainData.h \
	compression.h compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o$@ -c dataStorage/SlowData.cpp

$(ODIR)/decode.o: arduinoContact/decode.cpp arduinoContact/decode.h arduinoContact/Serial.h \
	compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/decode.cpp		
$(ODIR)/Serial.o: arduinoContact/Serial.cpp arduinoContact/Serial.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/Serial.cpp	

$(ODIR)/mainState.o: state/mainState.cpp state/mainState.h lamps/lamps.h
	$(CXX) $(CXXFLAGS) -pthread -o$@ -c state/mainState.cpp	

$(ODIR)/mainServer.o: httpServer/mainServer.cpp httpServer/mainServer.h \
	httpServer/pages/webGraph.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -o$@ -c httpServer/mainServer.cpp
$(ODIR)/telegramBot.o: telegramBot/telegramBot.cpp telegramBot/telegramBot.h \
	telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -o$@ -c telegramBot/telegramBot.cpp

$(ODIR)/webGraph.o: httpServer/pages/webGraph.cpp httpServer/pages/webGraph.h
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/webGraph.cpp

$(ODIR)/httpGetPostPut.o: lamps/httpGetPostPut.cpp lamps/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -o$@ -c lamps/httpGetPostPut.cpp	
$(ODIR)/mpd.o: mpd/mpd.cpp mpd/mpd.h
	$(CXX) $(CXXFLAGS) -o$@ -c mpd/mpd.cpp		

$(ODIR)/lamps.o: lamps/httpGetPostPut.h lamps/lamps.cpp lamps/lamps.h
	$(CXX) $(CXXFLAGS) -o$@ -c lamps/lamps.cpp

.PHONY : clean
clean:
	rm pi $(OBJS)
