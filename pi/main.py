#!/usr/bin/python3
################################################################################
#                                   Home Pi                                    #
#                               by David Kleingeld                             #
#                                                                              #
#This program manages the seperate processes used by the home automation system#
#see the diffrent files for the many functions and what they do                #
################################################################################
#libs
import multiprocessing
import threading
import time

#private libs
import data.access
import data.sensors
import data.analysisAndSaving
import functions.misc
import scenes

import startTasks

import httpResponder
import telegramBot
import status


if __name__ == '__main__':
       
    data.access.init() #load data from hard drive
    lightSceneQueue = status.lightSceneQueue
    sleeping = status.sleeping
    tasksQueue = startTasks.tasksQueue #TODO check if this can be done with import statment
    resourceLocks = startTasks.resourceLocks
    needs = startTasks.needs
    sensorRequest, sensorData = data.sensors.init()
    sensorGet, sensorGetBack, analysisRq = data.analysisAndSaving.init()
    
#   start the main process threads and close them down nicely if
#   we are ctrl-c -ed. All main processes are in separate files. 
#   processes send things that need to be done to the queue which starts the
#   processes in seperate threads

    #start Qeueu watch and execute script
    p1 = multiprocessing.Process(target=startTasks.processTasks, 
                                 args=(tasksQueue, resourceLocks, needs))
    p1.start()
    
    #start webserver
    p2 = multiprocessing.Process(target=httpResponder.httpResponder, 
                                 args=())
    p2.start()
    
    #start telegram bot
    p3 = multiprocessing.Process(target=telegramBot.HttpRecieveServer, 
                                 args=(tasksQueue, resourceLocks, sensorGet,
                                       sensorGetBack, sensorRequest,analysisRq,
                                       lightSceneQueue))
    p3.start()

    #start sensor shit
    p4 = multiprocessing.Process(target=data.sensors.read, 
                                 args=(sensorRequest, sensorData,
                                       lightSceneQueue, sleeping))
    p4.start()    

    #start database management     
    p5 = multiprocessing.Process(target=data.analysisAndSaving.process, 
                                 args=(sensorData, sensorGet, sensorGetBack,
                                       analysisRq, resourceLocks))
    p5.start()


    t1 = threading.Thread(target = status.lampManager,
                          args   = (lightSceneQueue, resourceLocks))
    t1.start()
        
    
    data.analysisAndSaving.sensorSchedual(sensorRequest)
#   start light things like scheduals that will run with GIL on this process    
    #TODO make this a general queue for all kind of things that need a schedual
    
    #TODO lamp status switch, takes some queue... yeah another one for overrides
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
