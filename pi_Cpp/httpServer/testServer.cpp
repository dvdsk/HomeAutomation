#include <thread>
#include <mutex>

#include "../telegramBot/telegramBot.h"
#include "../state/mainState.h"
#include "mainServer.h"
#include "../config.h"

std::shared_ptr<std::mutex> stop = std::make_shared<std::mutex>();
std::shared_ptr<TelegramBot> bot = std::make_shared<TelegramBot>();
std::shared_ptr<MainState> state = std::make_shared<MainState>();

void test(std::shared_ptr<MainState> state){
	state->thread_watchForUpdate();
}

int main(void)
{
	(*stop).lock();
	std::thread t1(test, state);
	std::thread t2(thread_Https_serv, stop, bot, state);

	getchar();
	//bot->test();
	state->runUpdate();
	getchar();
	//stop the server and watch thread
	state->shutdown();
	(*stop).unlock();
	
	t1.join();
	t2.join();

  return 0;
}

