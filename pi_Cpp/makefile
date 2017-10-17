MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

#CXX = g++-6
CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

CXXFLAGS += -DPRINT_STATE_MESSAGES


LDFLAGS = -L/usr/local/ssl/lib -L/usr/local/lib 
LDLIBS = -lssl -lcrypto

RASPBERRYPI=$(cat /proc/cpuinfo | grep "Hardware")
IFRASPBERRY=$(echo Hardware : BCM2835)

ifeq ($(RASPBERRYPI),$(IFRASPBERRY))
  LDLIBS += -lrf24
endif

ODIR = .BUILD
OBJS = $(addprefix $(ODIR)/,main.o decode.o Cache.o MainData.o PirData.o SlowData.o nodeMaster.o MainHeader.o  mainServer.o mainState.o telegramBot.o HttpSocket.o HttpsSocket.o commandline.o webGraph.o plotly.o dygraphs.o mpd.o lamps.o default.o SleepInterrupt.o GoingToSleep.o Minimal.o Wakeup.o sunSetRise.o minimalShell.o)


#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: homeAutomation
debug: CXXFLAGS += -DDEBUG -g
debug: homeAutomation
nothread: CXXFLAGS += -DNOTHREAD
nothread: homeAutomation


homeAutomation: $(OBJS)
	$(CXX) $(CXXFLAGS) -o homeAutomation $(OBJS) \
	-lmicrohttpd -lpthread -lmenu -lncurses \
	-lboost_system -lboost_thread $(LDFLAGS) $(LDLIBS)

$(ODIR)/main.o: main.cpp config.h arduinoContact/decode.h \
	dataStorage/MainData.h dataStorage/PirData.h dataStorage/SlowData.h \
	state/mainState.h telegramBot/telegramBot.h state/stateManagement.cpp\
	httpServer/mainServer.h arduinoContact/nodeMaster.h commandLine/commandline.h \
	debug.h smallFunct/HttpsSocket.h 
	$(CXX) $(CXXFLAGS) -o$@ -c main.cpp

$(ODIR)/Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/Cache.cpp
$(ODIR)/MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainHeader.cpp	
$(ODIR)/MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
  dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainData.cpp

$(ODIR)/PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/PirData.cpp
$(ODIR)/SlowData.o: dataStorage/SlowData.cpp dataStorage/SlowData.h dataStorage/MainData.h \
	compression.h compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/SlowData.cpp

$(ODIR)/decode.o: arduinoContact/decode.cpp arduinoContact/decode.h arduinoContact/radio.h\
	compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/decode.cpp		
$(ODIR)/nodeMaster.o: arduinoContact/nodeMaster.cpp arduinoContact/nodeMaster.h\
	arduinoContact/decode.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/nodeMaster.cpp	

$(ODIR)/mainState.o: state/mainState.cpp state/mainState.h lamps/lamps.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/mainState.cpp	
$(ODIR)/default.o: state/majorStates/default.cpp state/majorStates/default.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/default.cpp	 
$(ODIR)/SleepInterrupt.o: state/majorStates/SleepInterrupt.cpp state/majorStates/SleepInterrupt.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/SleepInterrupt.cpp	
$(ODIR)/GoingToSleep.o: state/majorStates/GoingToSleep.cpp state/majorStates/GoingToSleep.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/GoingToSleep.cpp	
$(ODIR)/Minimal.o: state/majorStates/Minimal.cpp state/majorStates/Minimal.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/Minimal.cpp	
$(ODIR)/Wakeup.o: state/majorStates/Wakeup.cpp state/majorStates/Wakeup.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/Wakeup.cpp	

$(ODIR)/mpd.o: mpd/mpd.cpp mpd/mpd.h
	$(CXX) $(CXXFLAGS) -o$@ -c mpd/mpd.cpp		
$(ODIR)/lamps.o: smallFunct/HttpSocket.h lamps/lamps.cpp lamps/lamps.h
	$(CXX) $(CXXFLAGS) -o$@ -c lamps/lamps.cpp

$(ODIR)/mainServer.o: httpServer/mainServer.cpp httpServer/mainServer.h \
	httpServer/pages/webGraph.h smallFunct/minimalShell.h 
	$(CXX) $(CXXFLAGS) -lmicrohttpd -o$@ -c httpServer/mainServer.cpp
$(ODIR)/plotly.o: httpServer/pages/plotly.h httpServer/pages/plotly.cpp
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/plotly.cpp
$(ODIR)/dygraphs.o: httpServer/pages/dygraphs.h httpServer/pages/dygraphs.cpp
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/dygraphs.cpp
$(ODIR)/webGraph.o: httpServer/pages/webGraph.cpp httpServer/pages/webGraph.h \
	httpServer/pages/dygraphs.h httpServer/pages/plotly.h config.h
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/webGraph.cpp

$(ODIR)/sunSetRise.o: smallFunct/sunSetRise.h smallFunct/sunSetRise.cpp 
	$(CXX) $(CXXFLAGS) -o$@ -c smallFunct/sunSetRise.cpp
$(ODIR)/minimalShell.o: smallFunct/minimalShell.h smallFunct/minimalShell.cpp 
	$(CXX) $(CXXFLAGS) -o$@ -c smallFunct/minimalShell.cpp
$(ODIR)/HttpSocket.o: smallFunct/HttpSocket.cpp smallFunct/HttpSocket.h
	$(CXX) $(CXXFLAGS) -o$@ -c smallFunct/HttpSocket.cpp	
$(ODIR)/HttpsSocket.o: smallFunct/HttpsSocket.cpp smallFunct/HttpsSocket.h
	$(CXX) $(CXXFLAGS) -o$@ -c smallFunct/HttpsSocket.cpp	

$(ODIR)/commandline.o: commandLine/commandline.cpp commandLine/commandline.h \
	config.h state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c commandLine/commandline.cpp

$(ODIR)/telegramBot.o: telegramBot/telegramBot.cpp telegramBot/telegramBot.h \
	telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -o$@ -c telegramBot/telegramBot.cpp


.PHONY : clean
clean:
	rm homeAutomation $(OBJS)
