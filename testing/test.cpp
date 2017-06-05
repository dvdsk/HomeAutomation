#include <string>
#include <ctime>
#include <cstdint> //uint16_t
#include <iostream>
#include <thread>
#include <atomic>

class dataClass{
	public:
		dataClass(){dataInt = 42;}
		int dataInt;	
};


class Parant {
	public:
		Parant(dataClass* data_){
			data= data_;
		}
		virtual ~Parant() = default;
		bool update(){
			if(data->dataInt < 42)
				return true;
			else{
				data->dataInt--; 
				return false;}
		}

	private:
		int a;
		int get() {return a;}

	protected:
		dataClass* data;
};



class Child_A : public Parant
{
	public:
		Child_A(dataClass* data_) : Parant(data_){std::cout<<"created child_A\n";	}
		~Child_A(){std::cout<<"removed child_A\n";}
		
	private:
		int a;
};

class Child_B : public Parant
{
	public:
		Child_B(dataClass* data_) : Parant(data_){std::cout<<"created child_B\n"; }
		~Child_B(){std::cout<<"removed child_B\n";}
	private:
		int a;
};

int main(){
	Parant* currentChild;

	dataClass* data = new dataClass;
	currentChild = new Child_A(data); 

	if(currentChild->update()){
		delete currentChild;
		currentChild = new Child_B(data); 
	}
	if(currentChild->update()){
		delete currentChild;
		currentChild = new Child_A(data); 
	}
	if(currentChild->update()){
		delete currentChild;
		currentChild = new Child_B(data); 
	}
	if(currentChild->update()){
		delete currentChild;
		currentChild = new Child_B(data); 
	}

	delete currentChild;
	delete data;
}







//class thread_Child : public Parant
//{
//	private:
//		int a;
//		std::atomic<bool> stop;
//		std::thread* id;


//		int get() {return a;}
//		static void* threadFunction(Child_A* arg) {
//			while(!arg->stop){}
//			std::cout<<"by\n";
//			return 0;}

//	public:
//		Child_A(){stop = false; id = new std::thread(threadFunction, this);	}
//		~Child_A() {stop = true; id->join();}

//};


//int main(int argc, char** argv) {
//	Parant* currentState = new Child_A();
//	delete currentState;
//	std::cout<<"deleted currentState 1\n";
//	currentState = new Child_A();
//	delete currentState;
//	std::cout<<"deleted currentState 2\n";
//	currentState = new Child_A();
//	delete currentState;
//	std::cout<<"deleted currentState 3\n";
//}
