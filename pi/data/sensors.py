#!/usr/bin/python3

import serial #http://pythonhosted.org/pyserial/
import time 
import queue
import threading
import numpy
import multiprocessing

import status
boudrate = 115200


def init():
    sensorRequest = multiprocessing.Queue()
    sensorData = multiprocessing.Queue()
    
    return sensorRequest, sensorData
    
def read(sensorRequest, sensorData, lightSceneQueue, sleeping):
    #read in analyse for short latency requests and ouput all data to buffer, 
    #then after 10 seconds put the buffer into a Queue to be further processed
    global prevMovement
    global peeing
    peeing = False
    prevMovement = int(time.time()) - 15
    
    with serial.Serial('/dev/ttyUSB0', boudrate, timeout = 2) as ser:
        time.sleep(10) #allow the arduino and raspberry pi to initialise
        ser.reset_input_buffer()
        while True:
            output = ser.readline()
            if not sensorRequest.empty():
                request = sensorRequest.get()
                ser.write(request)           
            if output != b'\n':  #if something happening not stdby
                process(output, sensorData, lightSceneQueue, sleeping)    
    return
    
def lamptrigger(lightSceneQueue, sleeping):
    global prevMovement
    global peeing
    now = time.time()
    if now - prevMovement > 15 and sleeping.is_set() and not peeing: #15 sec to walk back to bed
        print('started')
        #if the last movement was more then a minute ago (will later be replaced
        #with the demand for triggering a second motion sensor and user is 
        #sleeping)
        lightSceneQueue.put(status.goNightPee)
        peeing = True
    elif now - prevMovement > 5 and peeing and sleeping.is_set():
        print('done peeing')
        lightSceneQueue.put(status.doneNightPee)
        peeing = False
        
    prevMovement = now
    return

def checklights():
    pass
    return

def process(output, sensorData, lightSceneQueue, sleeping):
    #get new data from the queue and store it in a global variable and save it 
    #to the disk in binairy form
    if output == b'm\n':
        lamptrigger(lightSceneQueue, sleeping)
    elif output[0] == 108:#108 = acii code for l (this is L not one)
        checklights()
    else:
        sensorData.put(output)
    return

   
