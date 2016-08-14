#!/usr/bin/python3

import serial #http://pythonhosted.org/pyserial/
import time 
import queue
import threading

boudrate = 9600
devicePort = '/dev/ttyUSB0'

def init():
    pass
    
extraSensorTask = queue.Queue()

def timeReadSpeed(extraSensorTask):
    #read in analyse for short latency requests and ouput all data to buffer, 
    #then after 10 seconds put the buffer into a Queue to be further processed

    i = 0
    b = ['none']
    with serial.Serial('/dev/ttyUSB0', boudrate, timeout=2) as ser:
        time.sleep(4)
        ser.reset_input_buffer()
        program_starts = time.time()
        while True and i < 1000:
            if not extraSensorTask.empty():
                request = extraSensorTask.get()
#                print('sending request:',request)
                ser.write(request)
            output = ser.readline()
            if output != b'm0\n':  #if something happening not sensor stdby
                process(output)
            i += 1
    now = time.time()
    print("Time between updates: {0} milli second".format((now - program_starts)))
    return

def maxTimeBetwMovementSensor(extraSensorTask):
    #read in analyse for short latency requests and ouput all data to buffer, 
    #then after 10 seconds put the buffer into a Queue to be further processed
    def putFast(extraSensorTask):
        while True:
            extraSensorTask.put(b'00c')
            time.sleep(2)
        return

    t = threading.Thread(target = putFast, 
                         args   = (extraSensorTask,))
    t.start()


    i = 0
    deltaT = 0
    with serial.Serial('/dev/ttyUSB0', boudrate, timeout=2) as ser:
        time.sleep(4)
        ser.reset_input_buffer()
        t1 = time.clock()
        while True and i < 10:
            if not extraSensorTask.empty():
                request = extraSensorTask.get()
                ser.write(request)
            output = ser.readline()
#            print("output:",output)
            if b"\n" in output:
                t0 = t1
                t1 = time.clock()
                if t1-t0 > deltaT:
                    deltaT = t1-t0                  
            if b'h' in output: #if reply contains shit
                i += 1
    print("Max time between movement data recieved: {} milli seconds".format(deltaT*1000))
    return    

def timeSHT(extraSensorTask):
    #read in analyse for short latency requests and ouput all data to buffer, 
    #then after 10 seconds put the buffer into a Queue to be further processed
    def putFast(extraSensorTask):
        while True:
            extraSensorTask.put(b'00c')
        return

    t = threading.Thread(target = putFast, 
                         args   = (extraSensorTask,))
    t.start()


    i = 0
    with serial.Serial('/dev/ttyUSB0', boudrate, timeout=2) as ser:
        program_starts = time.time()
        while True and i < 10:
            if not extraSensorTask.empty():
                request = extraSensorTask.get()
#                print('sending request:',request)
                ser.write(request)
            output = ser.readline()
            print(output)
            if b"bufferlen" in output:
                print(output)            
            if b'h' in output: #if reply contains shit
                i += 1
    now = time.time()
    print("Time between SHT data: {0} second".format((now - program_starts)))
    return
    
            
def read(extraSensorTask):
    #read in analyse for short latency requests and ouput all data to buffer, 
    #then after 10 seconds put the buffer into a Queue to be further processed

    with serial.Serial('/dev/ttyUSB0', boudrate, timeout = 1) as ser:
        time.sleep(4) #allow the arduino to initialise
        ser.reset_input_buffer()
        while True:
            output = ser.readline()
            if not extraSensorTask.empty():
                request = extraSensorTask.get()
                ser.write(request)           
            print("output:",output)
            if output != b'm0\n':  #if something happening not sensor stdby
                process(output)    
    return
    
    
def lamptrigger():
#    print('lamps activated for a certain timeout')
    pass

def process(output):
    #get new data from the queue and store it in a global variable and save it 
    #to the disk in binairy form
    if output == b'm1\n':
        lamptrigger()
    elif b't' in output:#if the first part of output is letter t 
        print("temperature is:")
        print(output)
        pass
    return

    



def queueput(extraSensorTask):
#   low arduino load optimised code for requesting data:
#   diget of number defines if we are requesting sensor data
#   or controlling something (0 or 1), then a number for the sensor to request
#   this number can be 2 digets
    time.sleep(1.1)
    extraSensorTask.put(b'00')
    while True:
        time.sleep(0.1)
        extraSensorTask.put(b'00')
#    time.sleep(4)
#    extraSensorTask.put(b'00')
#    time.sleep(4)
#    extraSensorTask.put(b'01')
    return    

#timeSHT(extraSensorTask)
t = threading.Thread(target = read, 
                     args   = (extraSensorTask,))
t.start()
queueput(extraSensorTask)
#    

