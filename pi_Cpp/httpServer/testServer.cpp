#include <thread>
#include <mutex>

#include "../telegramBot/telegramBot.h"
#include "../state/mainState.h"
#include "mainServer.h"

std::shared_ptr<std::mutex> stop = std::make_shared<std::mutex>();
TelegramBot bot;

int main(void)
{

	(*stop).lock();
	std::thread t1(th_Https_serv, stop);

	getchar();
	(*stop).unlock();
	
	t1.join();

  return 0;
}

