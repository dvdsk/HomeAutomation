#!/usr/bin/python3
#manages all usefull data, sensor output goes here using queue's, and is fetched
#from programms that need data using queues. Basically its a hello I want Y and
#it gives the last version of Y from the Y queue

import time
import functions.strwrc as strwrc
import os.path
import json
import threading

#TODO reads and stores data to disk when needed
def init():
    def loadFromDisk(varName, default):
        print('loading:', '/home/pi/HomeAutomation/pi/data/'+varName+'.json')
        if os.path.isfile('/home/pi/HomeAutomation/pi/data/'+varName+'.json'):   
            with open('/home/pi/HomeAutomation/pi/data/'+str(varName)+'.json', 'r') as readfile:
                var = json.load(readfile)
        else:
            var = default
        return var

    #load date from disk    
    global permissions
    global codes
    
    print('loading data from disk')    
    permissions = loadFromDisk('permissions', {})  
    codes = loadFromDisk('codes', {})  
    
    #set temp vars in memory
    global replyData
    global strwrcData
    strwrcData = [0, None]
    replyData = {}

    return

def writeToFile(varName, data):
    with open('/home/pi/HomeAutomation/pi/data/'+str(varName)+'.json', 'w') as writefile:
        json.dump(data, writefile)

#DATA THAT NEED SAVING #commented to check if used #TODO remove from file 
#def sensorDbPut():
#    
#    dataCounter = 0
#    dataArray = np.full((100,3))
#    
#    return dataFromDb

#def sensorDbGetLine():
#    
#    return dataFromDb
    

def getNewData(resourceLocks, dataNeeded, sensorGet, sensorGetBack, sensorRequest):
    #TODO check if data in database not too old
    RCfromHR = {'temparature and humidity' :b'01',
                'lorum ipsum': b'01'}
    
    resourceLocks['sensors'].acquire(blocking=True, timeout=10)
    try:
        sensorGet.put(dataNeeded)
        sensorRequest.put(RCfromHR[dataNeeded])
        raw = sensorGetBack.get()
    except Exception as e:
        from config import bcolors
        print(bcolors.FAIL+'ERROR'+bcolors.ENDC+' while running: getNewData'
        +'\n'+'---->'+str(e))
    resourceLocks['sensors'].release()
    
    raw = raw[1::]
    if b"t" in raw:
        h = raw.index('h')#TODO CHECK THE FORMAT FOR THIS #TODO NO RATHER JUST GET IT FROM DB 
        T = raw[1:h].decode() #temp #TODO WITH WARNING IF DATA TOO OLD #KEEP THE FUNC FOR PLANT
        H = raw[7:12].decode()#humidity
        return T, H
    elif 'yolo' in raw:
        pass
        return


#VARS THAT NEED SAVING
"""codes"""
def getCodes():
    global codes
    
    toReturn = codes
    return toReturn    

def addCode(codeId, Permissions, timeOut):
#   here permissions is a list of allowed commands
#   timeout is the timeout for this collection of commands
#   usedBy is also a list of the users who have activated this key and gain 
#   commands from it (no use bothering the rest)
    global codes
    codes[codeId] = {'usedBy': [],
                     'Permissions': Permissions,
                     'timeOut': timeOut}
    #save to disk
    writeToFile('codes', codes)   
    return

def addCodeUser(codeId, userId):
#   update the list of users who uses a code

    global codes
    codes[codeId]['usedBy'].append(userId)
    
    #save to disk
    writeToFile('codes', codes)   
    return

def deactivateCode(codeId):
#   sets the timeout to 0 effectively disabling the code
    global codes
    
    codes[codeId]['timeOut'] = 0
    
    #save to disk
    writeToFile('codes', codes)
    return

"""permissions"""
def getPermissionsData():
    global permissions
    
    toReturn = permissions
    return toReturn    

def addPermissionsUser(userId, data):
    global permissions
    permissions[userId] = data
    #save to disk
    writeToFile('permissions', permissions)
    
    return    

def changePermissions(userId, plist, timeOut):
    global permissions
    
    if type(plist) is str:
        plist = [plist]
    
    for permission in plist:
        permissions[userId]['permissions'][permission] = timeOut
    
    #save to disk
    writeToFile('permissions', permissions)
    return

#TEMP VARS
"""strwrc"""
def getStrwrcData():
    global strwrcData
    
    toReturn = strwrcData
    return toReturn

def renewStrwrcData():
    global strwrcData

    strwrcData = [time.time(), strwrc.checkStrwLoad()]
    return strwrcData

"""replyData"""
def checkReplyDataAge(replyData):
    users_todelete = []
    
    #search for timed out items that need deletion
    for user in replyData.keys():
        for request in replyData[user].keys():
            #is it too old?
            if replyData[user][request]['time'] < int(time.time())-900:
                #delete in place if there are other requests of that user
                if len(replyData[user].keys()) > 1:
                    del replyData[user][request]
                #schedual for deleting the whole user when the iteration 
                #this to to prevent error due to changing dict size
                else:
                    users_todelete.append(user)
    
    for user in users_todelete:
        del replyData[user]
        
    return replyData
    
def getReplyData():
#   read some data, locking needs to be taken care off though. This is done
#   using startTasks.runLocked()

    #tell python we want the global replyData var and not make a local one
    global replyData 
    
    toReturn = replyData
    return toReturn
    
def addToReplyData(userId, replyToId, function, *funcArgs):
    global replyData    
    data = {replyToId: {'function': function,
                        'funcArgs': funcArgs,
                        'time'    : int(time.time())}}
                         
    replyData[userId] = data
    checkReplyDataAge(replyData)
    return
    
def deleteReplyData(userId, replyToId):
    global replyData
    if userId in replyData.keys():
        if replyToId in replyData[userId].keys():
            if len(replyData[userId].keys()) == 1:
                del replyData[userId]
            else:
                del replyData[userId][replyToId]
    return
