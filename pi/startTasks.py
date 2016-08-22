#!/usr/bin/python3

import functions.audio
import functions.computer
import functions.lamps
import functions.strwrc
from config import bcolors

import scenes

import multiprocessing
import threading



'''
Queue if filled with "q.put(function, arg1, arg2, arg3 ,etc)" can be executed with:

fA = q.get()
f = fA[0]
a = fA[1:]

f(*args)
'''

#functions that need a lock
needs_mpd = ['musicStatus','ebookInfo', 'Playback','musicOff',
             'addFromPlayList','goingToSleep','wakeup', 'jazz', 'testmpd',
             'leftHome','returnedHome']
needs_lamps = ['dimm','dimmMax','Off','On','SetPreset','turnOff','turnOn',
               'status']
needs = {'mpd': needs_mpd,
         'lamps': needs_lamps}

def init():
#   makes and returns the Queue that is used to pass functions to start
#   to this script and the list used to keep track of locked resources
#   this queue is not shared between threads but between this process and the
#   other processes
    tasksQueue = multiprocessing.Queue(maxsize=0)

    #these locks can only be aquired by one thread they are used to lock
    #needed resources to prevent crashing threads
    mpd             = threading.Lock() 
    lamps           = threading.Lock()
    replyData       = threading.Lock()
    strwrcData      = threading.Lock()
    permissionsData = threading.Lock()
    codesData       = threading.Lock()
    sensor          = threading.Lock()
    sDataFile       = threading.Lock()
    sensorRq        = threading.Lock()
    
    resourceLocks = {'mpd': mpd,
                     'lamps': lamps,
                     'replyData': replyData,
                     'strwrcData': strwrcData,
                     'permissionsData': permissionsData,
                     'codesData' : codesData,
                     'sensors': sensor,
                     'sensorDb': sDataFile,
                     'sensorRq': sensorRq}
    
    needs_mpd = ['musicStatus','ebookInfo', 'Playback','musicOff',
                 'addFromPlayList','goingToSleep','wakeup', 'jazz', 'testmpd',
                 'leftHome','returnedHome']
    needs_lamps = ['dimm','dimmMax','Off','On','SetPreset','turnOff','turnOn',
                   'status']
    
    needs = {'mpd': needs_mpd,
             'lamps': needs_lamps}
        
    return tasksQueue, resourceLocks, needs


def run(func,args):
#   start the function using a try exept to make sure the thread cant crash
    if args[0] is None:
        try:
            func()
        except Exception as e:
            print(bcolors.FAIL+'ERROR'+bcolors.ENDC+' while running:'
            ,func.__name__,'with args:',args,'\n'+'---->'+str(e))
    else:
        try:
            func(*args)
        except Exception as e:
            print(bcolors.FAIL+'ERROR'+bcolors.ENDC+' while running:'
            ,func.__name__,'with args:',args,'\n'+'---->'+str(e))
            
def runLocked(func, lock, *args):
#   run a function in try except way with locks (the try except to make sure we
#   unlock even if errors occur
    lock.acquire(blocking=True, timeout=10)
    output = None
    try:
        if len(args) == 0:
            output = func()
        else:
            output = func(*args)
    except Exception as e:
        print(bcolors.FAIL+'ERROR'+bcolors.ENDC+' while running:'
        ,func.__name__,'with args:',args,'\n'+'---->'+str(e))
    finally:
        lock.release()
    return output


def tryStart(funcName, func, args, resourceLocks, needs):
#   try to start a task. taskName is a list where the first element is the name of
#   the function that needs to be started and the following elements are the 
#   different paramaters. Tasks are subject to GIL.
    
    if funcName in needs['mpd']:
        if resourceLocks['mpd'].acquire(blocking=True, timeout=10):
            run(func,args)
            resourceLocks['mpd'].release()
        else:
            print('failed to aquire lock')
    
    elif funcName in needs['lamps']:
        if resourceLocks['lamps'].acquire(blocking=True, timeout=10):
            run(func,args)
            resourceLocks['lamps'].release()
        else:
            print('failed to aquire lock')
    else:
        print('starting:',funcName,'with arguments:',args)
        run(func,args)
    


def processTasks(tasksQueue, resourceLocks, needs):
#   get a task from the Queue as soon as its availible and try to execute it.
#   if the task can not be executed start a thread that checks if it can be
#   periodically for a set amount of time.
    
    while True:
        task = tasksQueue.get() 
        
        func = task[0]
        args = task[1:]
        funcName = func.__name__
        
        #start a thread that will try to start the process
        t = threading.Thread( target = tryStart, 
                              args   = (funcName, func, args, resourceLocks, needs))
        t.start()
        
tasksQueue, resourceLocks, needs = init()
