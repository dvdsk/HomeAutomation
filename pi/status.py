#!/usr/bin/python3
################################################################################
#                               Home Pi Responder                              #
#                               by David Kleingeld                             #
#                                                                              #
#This program responds to Http requests and facilitates communication between. #
#Smartphones running the tasker app and a linux home server                    #
################################################################################

import threading
import multiprocessing
import time
import random

import functions.lightColor as lightColor
import config
import phue
b = config.b

def init():
    lightSceneQueue = multiprocessing.Queue()
    sleeping = multiprocessing.Event()
    return lightSceneQueue, sleeping

def lampManager(lightSceneQueue, resourceLocks):
#   intial startup with default loop function, then check if a new lamp scene
#   is started, that scene's thread when the scene is done will also provide the
#   default loop function

    newRdy = threading.Event()
    prevDone = threading.Event()
    
    #default loop that manges colors throughout the day
    t = threading.Thread(target = colorLoopScene, 
                         args = (newRdy, prevDone, resourceLocks))
    t.start()
    
    while True:
        next = lightSceneQueue.get()#blocking
        print('lampmanager got new function')
        newRdy.set()
        
        prevDone.wait()#wait till the other thread signals its done
        newRdy.clear()#clear the newrdy (thus stop now other thread) signal
        prevDone.clear()
        
        #startScene
        t = threading.Thread(target = next, #advantage no need to pass arguments
                             args = (newRdy, prevDone, resourceLocks))
        t.start()
    return              


###FUNCTIONS NEEDED FOR LAMPSMANGER #TODO move all these to 'lamps.py'

def colorLoopScene(newRdy, prevDone, resourceLocks):
#   changes colors following pres set colortones for diffrent times of day does
#   not set the on or off switch of lamps this can still be manually overriden
    
    #set color with quick transition
    now = time.time()
    temperature = lightColor.getColorTemp(now)
    with resourceLocks['lamps']:
        command =  {'transitiontime' : 10, 'ct' : int(temperature)}
        b.set_light([1,2,3,4,5,6],command)

    if newRdy.wait(timeout=20): #if a new scene is waiting to start
        newRdy.clear()           #return
        prevDone.set()
        return
    
    while True:
        #update color every 5 minutes and let hue bridge interpolate 
        #within that timeframe
        now = time.time()
        temperature = lightColor.getColorTemp(now)
        with resourceLocks['lamps']:
            command =  {'transitiontime' : 300, 'ct' : int(temperature)}
            b.set_light([1,2,3,4,5,6],command)
        
        if newRdy.wait(timeout=300): #if a new scene is waiting to start
            newRdy.clear()           #return (used instead of sleep)
            prevDone.set()
            return
        
def someScene(newRdy, prevDone, resourceLocks):
#   loopy shit with instead of sleep: if event.wait(timeout=sleep):
#   immideatly clear the event and return, otherwise do more loopy loopy
    
    while True: #or some other condition
        print('doing fresh loop loopy stuff')
        if newRdy.wait(timeout=0.2): #if a new scene is waiting to start
            newRdy.clear()           #return
            prevDone.set()
            return
        print('still doing loopy stuff')
        if newRdy.wait(timeout=1): #if a new scene is waiting to start
            newRdy.clear()         #return
            prevDone.set()        
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit    
       
            
def evening(newRdy, prevDone, resourceLocks):
#   loopy shit with instead of sleep: if event.wait(timeout=sleep):
#   immideatly clear the event and return, otherwise do more loopy loopy

    print('starting evening')
    with resourceLocks['lamps']:    
        command =  {'transitiontime' : 1, 'ct' : 400, 'bri' : 255, 'on': True}
        b.set_light([1,2,3,4,5,6],command)
    
    if newRdy.wait(timeout=60*120): #if a new scene is waiting to start
        newRdy.clear()           #return
        prevDone.set()
        return
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit    

def night(newRdy, prevDone, resourceLocks):
#   loopy shit with instead of sleep: if event.wait(timeout=sleep):
#   immideatly clear the event and return, otherwise do more loopy loopy

    with resourceLocks['lamps']:   
        command =  {'transitiontime' : 1, 'ct' : 500, 'bri' : 180, 'on': True}
        b.set_light([1,2,3,4,5,6],command)
    
    if newRdy.wait(timeout=60*120): #if a new scene is waiting to start
        newRdy.clear()           #return
        prevDone.set()
        return
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit       

def bedlight(newRdy, prevDone, resourceLocks):
#   loopy shit with instead of sleep: if event.wait(timeout=sleep):
#   immideatly clear the event and return, otherwise do more loopy loopy

    with resourceLocks['lamps']:   
        command =  {'transitiontime' : 1, 'ct' : 500, 'bri' : 1, 'on': True}
        b.set_light([1,2,3,4,5,6],command)
    
    if newRdy.wait(timeout=60*120): #if a new scene is waiting to start
        newRdy.clear()           #return
        prevDone.set()
        return
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit   
            
def randoms(newRdy, prevDone, resourceLocks):
#   loopy shit with instead of sleep: if event.wait(timeout=sleep):
#   immideatly clear the event and return, otherwise do more loopy loopy
    with resourceLocks['lamps']:
        command = {'transitiontime' : 0}
        b.set_light([1,2,3,4,5,6], command)
    
    brightness = [1, 255]
    on = [True, True, False]
    
    random.choice(brightness)
    while True: #or some other condition
        with resourceLocks['lamps']:
            for light in [1,2,3,4,5,6]:
                command = {'bri': random.choice(brightness), 
                           'on': random.choice(on),
                           'xy': [random.random(),random.random()]}
                b.set_light(light, command)
                if newRdy.wait(timeout=0.05): #if a new scene is waiting to start
                    #deconstructor
                    command =  {'on' : True, 'bri' : 255}
                    b.set_light([1,2,3,4,5,6],command)
                    newRdy.clear()
                    prevDone.set()
                    return    
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit             

def wakeup(newRdy, prevDone, resourceLocks):
    Time_ = int(config.wakeupPeriod*600)
    
    with resourceLocks['lamps']:
        command = {'on' : True, 'bri': 0, 'ct': 500}
        b.set_light([2,3,4,5,6], command)
        command =  {'transitiontime' : Time_, 'on' : True, 'bri' : 255, 'ct' : 215}
        b.set_light([2,3,4,5,6], command)

    
    if newRdy.wait(timeout=60*120): #if a new scene is waiting to start
        newRdy.clear()              #return, 120 minutes allowes for an 
        prevDone.set()              #maximum continues day light setting of
        return                      #2 hours before 6 O clock (current default)
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit   



#TODO being developed
def goNightPee(newRdy, prevDone, resourceLocks):
#   put on lights on lowest level for mid night peeing, or maybe snacking? 
#   BAD BAD USER!!!! no eeating at night!

    with resourceLocks['lamps']:   
        command =  {'transitiontime' : 0, 'ct' : 500, 'bri' : 1, 'on': True}
        b.set_light([1,2,3,4,5,6],command)
    
    if newRdy.wait(timeout=20*60): #after 20 minutes you are pooppeeing to long
        newRdy.clear()             
        prevDone.set()
        return
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit   

def doneNightPee(newRdy, prevDone, resourceLocks):
#   turns lights back off and starts the normal color loop

    if newRdy.wait(timeout=15): #walk back to bed time
        newRdy.clear()          #return
        prevDone.set()
        return
    
    #TODO maybe change the color temp to show lamps will turn off?
    with resourceLocks['lamps']:   
        command =  {'on': False}
        b.set_light([1,2,3,4,5,6],command)
    
    colorLoopScene(newRdy, prevDone, resourceLocks)#default color loop shit
            
lightSceneQueue, sleeping = init()
