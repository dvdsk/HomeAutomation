#!/usr/bin/python3
################################################################################
#                               Home Pi Responder                              #
#                               by David Kleingeld                             #
#                                                                              #
#This program responds to Http requests and facilitates communication between. #
#Smartphones running the tasker app and a linux home server                    #
################################################################################
import requests
import time
import datetime
from time import sleep
import pyotp #used for time based auth codes
import json
import string
import queue
import threading
import copy
import editdistance
import pafy
import os
import random
import pyshorteners
from multiprocessing import Process


import functions.lamps as lamps
import functions.misc as misc
import functions.audio as audio
import startTasks
import config
import scenes
import status as lightstatus
import data.access as access

   
def sendrpl(message, reply):
    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': reply,
                   'reply_to_message_id': message['message_id']}
    
    send        = requests.post("https://api.telegram.org/bot"+config.token+
                                "/sendMessage", params=botmessage)
    return 

"""AUTHENTICATION AND USER MANAGMENT"""
def newUser(message, resourceLocks):

    data = copy.copy(message['from'])
    del data['id']
    
    data['permissions'] = {}
    
    
    startTasks.runLocked(access.addPermissionsUser, 
                         resourceLocks['permissionsData'], 
                         message['from']['id'], data) 
    return


def checkPerm(message, resourceLocks, commandName):
    #get the data
    permData = startTasks.runLocked(access.getPermissionsData, 
                                    resourceLocks['permissionsData'])                         
    userId = str(message['from']['id'])
    #check if user in the database
    if userId in permData.keys():
        #check if function is currently allowed
        if commandName in permData[userId]['permissions'].keys():
            #check if permission is still valid
            TimeOut = permData[userId]['permissions'][commandName]
            if TimeOut == -1 or TimeOut > int(time.time()):
                return True
    
    else:
        newUser(message, resourceLocks)
    sendrpl(message, 'access restricted! you need permissions from the '+
                     'supreme leader (@DeviousD) to acces this functionallity.')#TODO @DeviousD
    return False



def inputUser_1(message, permData, resourceLocks, funcArgs, errorTxt):
#takes the message obj, the permissions database and resourceLocks and the
#function with arguments it should then execute. Then tries to identify the user
#meant and executes the function with the message object as the first argument
#then the user
    def checkName(name):
    #   check if the name might occure in the database mistyped and return 
    #   the probable userId
        possibleUsers = [] #users that match assuming typos
        matchingUsers = [] #user(s) that match exactly
        for userId in permData.keys():
            for key in ['last_name', 'first_name','username']:
                if key in permData[userId].keys():
                    #find the number of characters in commen caps do not matter
                    a = permData[userId][key].lower()
                    b = name.lower()
                    n = editdistance.eval(a, b)

                    if n == 0 and userId not in matchingUsers:
                        matchingUsers.append(userId)
                    elif n < 2 and userId not in possibleUsers:
                        possibleUsers.append(userId)

        if len(matchingUsers) > 0:
            return True, matchingUsers
        elif len(possibleUsers) > 0:
            return False, possibleUsers
        return False, False

    def error():
        sendrpl(message, errorTxt)

    def presentUsersOptions(userIds, matching, message):
        #make the keyboard options and ask the user to click the name (s)he meant
        #from the options
        userReference = []
        print("userIds:",userIds)
        for ID in userIds:
            if 'first_name' in permData[ID]:
                if 'last_name' in permData[ID]:
                    userReference.append(permData[ID]['first_name']+' '+
                                       permData[ID]['last_name']+' ('+
                                       str(ID)+')')
                else:
                    if 'username' in permData[ID]:
                        userReference.append(permData[ID]['first_name']+' @'+
                                           permData[ID]['username']+' ('+
                                           str(ID)+')')
                    else:
                        userReference.append(permData[ID]['first_name']+' ('+
                                           str(ID)+')')
            elif 'last_name' in permData[ID]:
                    if 'username' in permData[ID]:
                        userReference.append(permData[ID]['last_name']+' @'+
                                           permData[ID]['username']+' ('+
                                           str(ID)+')')
                    else:
                        userReference.append(permData[ID]['last_name']+' ('+
                                           str(ID)+')')
            else:
                if 'username' in permData[ID]:
                    userReference.append(' @'+ permData[ID]['username']+
                                       ' ('+str(ID)+')')    
                             
        keyboard = ([userReference, ['cancel']])        
        if matching is True: #TODO if matching naar boven en keyboard in deze condidtion
            reply = (str(len(userReference))+' matching users have been found.')
        else:
            reply = ('no matching user was found,\n'+str(len(userReference))+
                    ' possible users have been found')
        
        botmessage  = {'chat_id': message['chat']['id'], 
                       'text': reply,
                       'reply_markup': json.dumps({"selective": True,
                                                   "keyboard": keyboard,
                                                   "one_time_keyboard": True}), 
                       'reply_to_message_id': message['message_id']}
        
        send        = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)   
        
        return send.json()['result']['message_id']
    #/presentUsersOptions

    userL = message['text'].split(' ', 1)
    if len(userL) < 2:
#        sendrpl(message, 'please input a username or id after the command')
        error()
        return
    else:
        user = userL[1]


    print("user:",user)
    #try if userId is given
    if user.isdigit(): #is the user id
        if user in permData.keys():
            userId = user 
            func = funcArgs[0]
            args = funcArgs[1]
            if not args:
                func(userId, message)
            else:
                func(userId, message, args)
        else:
             error()

    #try if first or last name is given
    else:
        results = checkName(user)
        if results[1] is not False: #if matching users
            if results[0] == True:
                if len(results[1]) == 1: #if only one user matches 
                    userId = results[1][0]
                    func = funcArgs[0]
                    args = funcArgs[1]
                    if not args:
                        func(userId, message)
                    else:
                        func(userId, message, args)
                else:
                    outgoingId = presentUsersOptions(results[1], results[0], message)
                    startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                                         message['from']['id'], outgoingId, 
                                         inputUser_2, message, permData, resourceLocks,
                                         funcArgs)
            else:
                outgoingId = presentUsersOptions(results[1], results[0], message)
                startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                                     message['from']['id'], outgoingId, 
                                     inputUser_2, message, permData, resourceLocks,
                                     funcArgs)
        else:
            error()
    return

def inputUser_2(message, orgMessage, permData, resourceLocks, funcArgs):
#expects a user id on the end following this structure: 
#David Kleingeld (15997283), note message is here the origional message
    a = message['text'].find('(')+1
    b = message['text'].find(')')
    user = message['text'][a:b]

    #try if userId is given
    if user.isdigit(): #is the user id
        if user in permData.keys():
            func = funcArgs[0]
            args = funcArgs[1]
            if not args:
                func(user, orgMessage)
            else:
                func(user, orgMessage, args)
        else:
             error(message)
    return


def showPerm(message, resourceLocks):      
    permData = startTasks.runLocked(access.getPermissionsData, 
                                resourceLocks['permissionsData'])    
    
    def printPerm(idList, message, permData):
        reply = ''
        if type(idList) is not list:
            idList = [idList]
        for ID in idList:
            #find out how to refer to this user
            if 'first_name' in permData[ID]:
                if 'last_name' in permData[ID]:
                    name = (permData[ID]['first_name']+' '+
                            permData[ID]['last_name']+' ('+
                            str(ID)+')')
                else:
                    if 'username' in permData[ID]:
                        name = (permData[ID]['first_name']+' @'+
                                permData[ID]['username']+' ('+
                                str(ID)+')')
                    else:
                        name = (permData[ID]['first_name']+' ('+
                                str(ID)+')')
            elif 'last_name' in permData[ID]:
                    if 'username' in permData[ID]:
                        name = (permData[ID]['last_name']+' @'+
                                permData[ID]['username']+' ('+
                                str(ID)+')')
                    else:
                        name = (permData[ID]['last_name']+' ('+
                                str(ID)+')')
            elif 'username' in permData[ID]:
                name = (' @'+ permData[ID]['username']+
                        ' ('+str(ID)+')')
            else:
                name = '('+str(ID)+')'
            
            #buildup permissions list for this user
            permissions= ''
            for funcName, timeout in permData[ID]['permissions'].items():
                if timeout == -1: #(-1 means no timeout) 
                    permissions = permissions+str(funcName)+'\n'
                else: 
                    timeTillTimeout = timeout - int(time.time())
                    if timeTillTimeout > 0:
                        if timeTillTimeout > 3600:
                            dt = datetime.fromtimestamp(0)
                            permissions = (permissions+str(funcName)+
                            'Valid until: {1}, {2}, {3}'.format(dt)+'\n')
                        else:
                            permissions = (permissions+str(funcName)+
                            'Valid for the next: '+str(int(timeTillTimeout/60))+
                            'minutes\n')                      
    
            #send total message
            reply = reply+'\n'+name+':\n'+permissions
            
            botmessage  = {'chat_id': message['chat']['id'], 
                           'text': reply, 'reply_to_message_id': message['message_id']}
               
            send        = requests.post("https://api.telegram.org/bot"+config.token+
                                        "/sendMessage", params=botmessage)
    
    args = [printPerm, permData]
    errorTxt = ("Format: '/showPerm u'\n"+
                "\t u: first, last or user-name")
    inputUser_1(message, permData, resourceLocks, args, errorTxt)
    return

def changePerm(message, resourceLocks):
    permData = startTasks.runLocked(access.getPermissionsData, 
                                resourceLocks['permissionsData'])   

    def set_2(message, resourceLocks, userId, permission):
    #   this is the second step in adding permissions, see the first one below
        timeOut = 0
        if message['text'].isdigit():
            timeOut = int(message['text'])
        elif str(message['text']) == '-1':
            timeOut = -1
        else:
            numb, unit,  = message['text'].split(' ')
            print(unit)
            if unit == 'min' or unit == 'minutes' or unit == 'minute':
                multiplier = 60
            elif unit == 'hour' or unit == 'hrs' or unit == 'hours':
                multiplier = 3600
            elif unit == 'day' or unit == 'days':
                multiplier = 86400
            else:
                sendrpl(message, 'incorrect time format')     
                multiplier = 0
            timeOut = int(float(numb)*multiplier+time.time())
        
        #check if user in the database
        if userId not in permData.keys():
            newUser(message, resourceLocks)
        
        startTasks.runLocked(access.changePermissions, 
                            resourceLocks['permissionsData'], userId, 
                            permission, timeOut) 
        return 

    def set_1(userId, message, args):
        permData = args[0]
        permission = args[1]
    #   first step in adding permissions
    
        botmessage  = {'chat_id': message['chat']['id'], 
                       'text': ('Changing the authorisation for the function: '
                                +str(permission)+' to add authorisation forever enter'
                                +'-1, to revoke authorisation enter 0 and to give '+
                                'authorisation temporairly enter the time till '+
                                'revoke (format: xx minutes, xx hours , xx days or '+
                                'xx weeks'),
                       'reply_markup': json.dumps({"force_reply":True,"selective": True}), 
                       'reply_to_message_id': message['message_id']}
    
        send        = requests.post("https://api.telegram.org/bot"+config.token+
                                "/sendMessage", params=botmessage)

        
        outgoingId  = send.json()['result']['message_id']
        startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                             message['from']['id'], outgoingId, 
                             set_2, resourceLocks, userId, permission)

    #permission to add
    a = message['text'].rfind(' ')
    permission = message['text'][a:].strip(' ')
    message['text'] = message['text'][:a]
    
    args = [permData, permission]
    funcArgs = [set_1, args]
    errorTxt = ("Format: '/changePerm u f'\n"+
                "\t u: first, last or user-name\n"+
                "\t f: an availible function or command")
    inputUser_1(message, permData, resourceLocks, funcArgs, errorTxt)
    return

  
def help(message, resourceLocks): 
    permData = startTasks.runLocked(access.getPermissionsData, 
                                resourceLocks['permissionsData'])    
    
    permissions= ''
    ID = str(message['from']['id'])
    
    if ID in permData.keys():
        
        #find out how to refer to this user
        if 'first_name' in permData[ID]:
            if 'last_name' in permData[ID]:
                name = (permData[ID]['first_name']+' '+
                        permData[ID]['last_name'])
            else:
                if 'username' in permData[ID]:
                    name = (permData[ID]['first_name']+' @'+
                            permData[ID]['username'])
                else:
                    name = (permData[ID]['first_name'])
        elif 'last_name' in permData[ID]:
                if 'username' in permData[ID]:
                    name = (permData[ID]['last_name']+' @'+
                            permData[ID]['username'])
                else:
                    name = (permData[ID]['last_name'])
        elif 'username' in permData[ID]:
            name = (' @'+ permData[ID]['username'])
        else:
            name = '('+str(ID)+')'
        
        #buildup permissions list for this user
        for funcName, timeout in permData[ID]['permissions'].items():
            if timeout == -1: #(-1 means no timeout) 
                permissions = permissions+str(funcName)+'\n'
            else: 
                timeTillTimeout = timeout - int(time.time())
                if timeTillTimeout > 0:
                    if timeTillTimeout > 3600:
                        dt = datetime.datetime.fromtimestamp(timeout)
                        permissions = (permissions+str(funcName)+
                        '\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t'+
                        '(Valid until: {:%Y-%m-%d %H:%M}'.format(dt)+')\n')
                    elif timeTillTimeout > 60:
                        permissions = (permissions+str(funcName)+
                        '\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t'+
                        '(Valid for the next: '+str(int(timeTillTimeout/60))+#TODO always round up
                        ' minutes)\n')
                    else:
                        permissions = (permissions+str(funcName)+
                        '\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t'+
                        '(Valid for less then a minute)\n')                      
    #/endif
    else:
        if 'first_name' in message['from']:
            if 'last_name' in message['from']:
                name = (message['from']['first_name']+' '+
                        message['from']['last_name'])
            else:
                if 'username' in message['from']:
                    name = (message['from']['first_name']+' @'+
                            message['from']['username'])
                else:
                    name = (message['from']['first_name'])
        elif 'last_name' in message['from']:
                if 'username' in message['from']:
                    name = (message['from']['last_name']+' @'+
                            message['from']['username'])
                else:
                    name = (message['from']['last_name'])
        elif 'username' in message['from']:
            name = (' @'+ message['from']['username'])
        else:
            name = '('+str(ID)+')'

    #send total message
    if permissions == '':
        reply = ('Hello '+name+', I am sorry this is a private bot,\nif you '+
                'have an activation key you can use /key followed by your 6 '+
                'digit key to gain access to some of this bots functionality')
    else:
        reply = ('Hello '+name+', you currently have access to the following '+
                'commands:\n\n'+permissions)
    
    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': reply, 'reply_to_message_id': message['message_id']}
       
    send        = requests.post("https://api.telegram.org/bot"+config.token+
                                "/sendMessage", params=botmessage)
    return
       
    
def lamps_1(message, tasksQueue, resourceLocks, lightSceneQueue):
    def getKeyboardOptions():
    #format keyboard options
        options = ['Ceiling Lamp On', 'Kitchen Lamp On', 'Desk Lamp On', 'other lamp On']
        for idx, lampOn in enumerate(lamps.status()):
            if lampOn:
                options[idx] = str(options[idx][:-2]+'Off')
        return options

    #check the lamps status and format the keyboard recourselocked
    options = startTasks.runLocked(getKeyboardOptions, resourceLocks['lamps'])

    keyboard = ( 
        [
                  [options[0], options[1]],
                  [options[2], options[3]],
          [ "all On"  , "advanced" , "all Off"   ],
        ] )
                
    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': 'Please choose a lamp to turn Off or On',
                   'reply_markup': json.dumps({"selective": True,
                                               "keyboard": keyboard,
                                               "one_time_keyboard": True}), 
                   'reply_to_message_id': message['message_id']}
    
    send        = requests.post("https://api.telegram.org/bot"+config.token+
                                "/sendMessage", params=botmessage) 

    outgoingId  = send.json()['result']['message_id']
    startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                         message['from']['id'], outgoingId, 
                         lamps_2, tasksQueue, resourceLocks, lightSceneQueue)
    return

def lamps_2(message, tasksQueue, resourceLocks, lightSceneQueue):
    if message['text'] == 'all On':
        tasksQueue.put([lamps.On , None])
    elif message['text'] == 'all Off':
        tasksQueue.put([lamps.Off , None])
    elif message['text'] == 'advanced':
        #keyboard settings
        keyboard = ( 
            [
              ["evening", "night"],
              ["bedlight","randoms"]
            ] )
        
        botmessage  = {'chat_id': message['chat']['id'], 
                       'text': 'Choose a lamp scene, randomLoop toggles',
                       'reply_markup': json.dumps({"selective": True,
                                                   "keyboard": keyboard,
                                                   "one_time_keyboard": True}), 
                       'reply_to_message_id': message['message_id']}
        
        send        = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)
                                
        outgoingId  = send.json()['result']['message_id']
        startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                             message['from']['id'], outgoingId, lamps_3, lightSceneQueue)
    
    else: #since keyboard options only no need for another if
        if 'Ceiling Lamp' in message['text']:
            lamp = 4
        elif 'Kitchen Lamp' in message['text']:
            lamp = 5                          
        else:
            lamp = 2
        
        if message['text'][-2:] == 'On':
            tasksQueue.put([lamps.turnOn, lamp])
        else:
            tasksQueue.put([lamps.turnOff, lamp])
    return

def lamps_3(message, lightSceneQueue):
#   sends the preset request forward to the setPreset function that
#   checks the text.
    if message['text'] == "evening":
        lightSceneQueue.put(lightstatus.evening)
    elif message['text'] == "night":
        lightSceneQueue.put(lightstatus.night)
    elif message['text'] == "bedlight":
        lightSceneQueue.put(lightstatus.bedlight)    
    elif message['text'] == "randoms":
        print('putting randoms')
        lightSceneQueue.put(lightstatus.randoms)        
    return


def status(message, resourceLocks, sensorGet, sensorGetBack, sensorRequest):
    def getLamps():
    #   run locked and get lamp status data
        lampstat = ['Ceiling Lamp Off', 'Kitchen Lamp Off', 'Desk Lamp Off', 'other lamp Off']
        for idx, lampOn in enumerate(lamps.status()):
            if lampOn:
                lampstat[idx] = str(lampstat[idx][:-3]+'On')  
        return lampstat
    
    #check the lamps status and format the keyboard recourselocked
    lampstat = startTasks.runLocked(getLamps, resourceLocks['lamps'])

    lampStat    = ('lamps: \n    '+
                   '\n    '.join(str(x) for x in lampstat))
    
    load        = misc.systemStatus()
    sysStat     = ('System load in the last: \n'+
                   '    minute: '    +str(int(load[0]*100))+'%\n'+
                   '    5 minutes: ' +str(int(load[1]*100))+'%\n'+
                   '    15 minutes: '+str(int(load[2]*100))+'%\n')
    
    musicStat = startTasks.runLocked(audio.musicStatus, resourceLocks['mpd']) 
    
    temp, humid = access.getNewData(resourceLocks, 'temparature and humidity', 
                                    sensorGet, sensorGetBack, sensorRequest)
    
    
    
    reply       = ('Sound: '+musicStat+'\n'+
                   'Volume: '+audio.volume()+'\n\n'+
                   'Lighting: '+lampStat+'\n\n'+
                   sysStat+'\n'+
                   'Temperature: '+str(temp)+'℃\n'+
                   'Humidity: '+str(humid)+'%\n')                   

    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': reply, 
                   'reply_to_message_id': message['message_id']}

    send        = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)
    return



def strwrc(message, resourceLocks):
#   check strw pcroom 4th floor for usage and reply sorted list    

    #get cached data:
    data = startTasks.runLocked(access.getStrwrcData, 
                                resourceLocks['strwrcData'])
    
    #check if still new enough else renew
    if data[0] < (time.time() - config.minDeltaT_strwRC):
        reply = 'Checking sterrenwacht pczaal load, this might take up to 10 seconds'
        botmessage  = {'chat_id': message['chat']['id'], 
                       'text': reply, 
                       'reply_to_message_id': message['message_id']}

        send        = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)
        
        data = startTasks.runLocked(access.renewStrwrcData, 
                                    resourceLocks['strwrcData'])
        
    #send result to client
    replytxt = ('This is a list of sterrenwacht hosts in the pc room on '+
             'the fourth floor of the Huygens building sorted on load:\n\n')

    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': str(replytxt+data[1]), 
                   'entities': json.dumps([{'type': 'pre',
                                            'offset': len(replytxt),
                                            'length': len(data[1])}]),
                   'reply_to_message_id': message['message_id']}

    send        = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)
                                    
    return


def wakeUp_1(message, resourceLocks, tasksQueue):
    #keyboard settings
    keyboard = ( 
        [
          [ "Initialise"  , "Cancel"   ],
        ] )
                               
    botmessage      = {'chat_id': message['chat']['id'], 
                       'text': 'Please confirm immidiate wakeup initialisation',
                       'reply_markup': json.dumps({"keyboard":keyboard,
                                                   "one_time_keyboard":True,
                                                   "selective": True}) , 
                       'reply_to_message_id':message['message_id']}
    
    send            = requests.post("https://api.telegram.org/bot"+config.token+
                                    "/sendMessage", params=botmessage)   
    
    outgoingId = send.json()['result']['message_id']
    startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                         message['from']['id'], outgoingId, 
                         wakeUp_2, tasksQueue)
    return

def wakeUp_2(message, tasksQueue):
    replyId   = message['message_id']
    if message['text'] == 'Initialise':
        reply = 'Initialised wakeup sequence'
        tasksQueue.put([scenes.wakeup, None])
    else:
        reply = 'Cancelling request'
    sendrpl(message, reply)
    return


def broadcast_1(message, tasksQueue, resourceLocks):   
    botmessage  = {'chat_id': message['chat']['id'], 
                   'text': 'Please give voice input',
                   'reply_markup': json.dumps({"force_reply":True,"selective": True}), 
                   'reply_to_message_id': message['message_id']}

    send        = requests.post("https://api.telegram.org/bot"+config.token+
                            "/sendMessage", params=botmessage)
    
    outgoingId  = send.json()['result']['message_id']
    startTasks.runLocked(access.addToReplyData, resourceLocks['replyData'],
                         message['from']['id'], outgoingId, 
                         broadcast_2, resourceLocks)

def broadcast_2(message, resourceLocks):
       
    if 'voice' in message.keys():
        file_id = message['voice']['file_id']
        duration = message['voice']['duration']
        updateargs = {'file_id': file_id}
        r = requests.get("https://api.telegram.org/bot"+config.token+"/getFile", params=updateargs)
        data = r.json()	#JSON conversion
        file_path = data['result']['file_path']
        streamingLink = ('https://api.telegram.org/file/bot'+str(config.token)+'/'+str(file_path))

        status, songId = startTasks.runLocked(audio.playStream, resourceLocks['mpd'], streamingLink)       
        sleep(duration+0.5)#wait till done playing      
        startTasks.runLocked(audio.restorePrev, resourceLocks['mpd'], status, songId)
    return


def streamAudio(message, resourceLocks, islink):
    #TODO implement adding to queue FIX LONG SONGS TAKING FOREVER TO START
    #maybe a buffer problem for mpd?
    
    if islink:
        url = message['text']
    else:
        url = message['text'].split(' ')[1]
    
    #try using the less reliable but faster internal backend first 
    #then fall back to youtube-dl if it fails
    try:
        os.environ["PAFY_BACKEND"] = "internal"
        video = pafy.new(url)
    except:
        os.environ["PAFY_BACKEND"] = ""    
        video = pafy.new(url)

    audiostreams = video.audiostreams
    badExtensions = ['opus', 'ogg', 'webm']
    bestBitrate = 0
    for a in audiostreams:
        if a.extension not in badExtensions:
            bitrate = int(a.bitrate[:-1])
            if bitrate > bestBitrate:
                bestBitrate = bitrate
                bestStream = a
            
    if bestBitrate > 0:
        streamingLink = bestStream.url         
        timestr = video.duration
        print('trying:',bestStream.extension, bestStream.bitrate)
                
        #do fancy list comprehension to convert timestring to seconds without
        #having to import datetime or do lots of lines credits: 
        #http://stackoverflow.com/questions/10663720/converting-a-time-string-to-seconds-in-python
        ftr = [3600,60,1]
        duration = sum([a*b for a,b in zip(ftr, map(int,timestr.split(':')))])

        status, songId = startTasks.runLocked(audio.playStream, 
                                              resourceLocks['mpd'], streamingLink)       
        sleep(duration+0.5)#wait till done playing      
        startTasks.runLocked(audio.restorePrev, resourceLocks['mpd'], status, songId)    
    return    
    

def graph(message, resourceLocks, analysisRq):
#   if paramaters are given in the format:
#   'time vs temperature/humidity/brightness, temperature/humidity/brightness, 
#    temperature/humidity/brightness last 5 hours/days/minutes' makes a graph
#   of that data and sends that back as images. TODO otherwise gives options
#   using a keybaord

    def error():
        sendrpl(message, "Format: '/graph x, vs y(1), y(2) ... last n U'\n"+
                "\tx: time/temperature/humidity/light \n"+
                "\ty(n): time/temperature/humidity/light\n"+
                "\tn: a number\n"+
                "\tU: hours/days/minutes"+
                "\toptional: 'type=line/scatter/histogram')")
        return
    
    def requestAndSendGraph(rq):
        resourceLocks['sensorRq'].acquire()
        analysisRq.put(rq)
        result = analysisRq.get()
        resourceLocks['sensorRq'].release()
        
        if result:
            botmessage  = {'chat_id': message['chat']['id'],
                           'reply_to_message_id': message['message_id']}
            
            send = requests.post("https://api.telegram.org/bot"+config.token+
                   "/sendPhoto", params=botmessage, files={'photo': open('/home/pi/bin/homeAutomation/data/graph.png', 'rb')})
        return
    
    plotOptions = ['time','temperature','humidity','light','co2']
    histOptions = ['temperature','humidity','light','co2']
    typePlotOpt = ['line','scatter']
    
    typePlot = None
    string = message['text'][7:]
    if 'type' in string:
        string, typePlot = string.split("type=", 2)
        typePlot = typePlot
    
    try: #use try except to check if correct format is adhered to
        toGraph, times = string.split(" last ", 2)

        #get start and stop time in seconds
        for u, F in zip(['seconds','minutes','hours','days'], [1,60,3600,3600*24]):
            if ' '+u in times:
                secondsBack = float(times.split(' '+u)[0] )*F
        
        t2 = int(time.time())
        t1 = t2-secondsBack
        
        #is it a histogram?
        print(toGraph.rstrip(" "))
        if toGraph.rstrip(" ") in histOptions:
            typePlot = 'histogram'
            x = toGraph.rstrip(" ")          
            rq = [x, ['time'], typePlot, int(t1),int(t2)]
            requestAndSendGraph(rq)
            return
        
        #get things to graph
        x, y = toGraph.split(" vs ", 2)
        if ',' in y and 'and' in y:
            ysplitted = y.split(", ")
            
            yList = [ysplitted[0] ]
            idx = 1
            while "and" not in ysplitted[idx]:
                yList.append(ysplitted[idx])
                idx += 1
                        
            yList = yList + (ysplitted[idx].split(" and ") )
        elif ',' in toGraph:
            yList = y.split(", ")
        else:
            yList = y.split(" and ")
        
    except Exception as e:
        error()
        print(e)
        return
    
    if typePlot == None:
        typePlot = 'line'
       
    #check if all arguments are correct
    if x not in plotOptions or y == x or typePlot not in typePlotOpt:
        error()
        return
    else:
        for y in yList:
            if y not in plotOptions:
                error()
                return
     
    rq = [x, yList, typePlot, int(t1),int(t2)]
    requestAndSendGraph(rq)
    return




def addKey(message, resourceLocks):   
    times = {'minutes': 60, 'hours': 3600, 'days':86400, 'weeks': 604800,
             'months': 2629743, 'minute': 60, 'hour': 3600, 'day':86400, 
             'week': 604800, 'month': 2629743}
    
    messageList = message['text'].split(' ')
    
    if len(messageList) > 2:
        codeId = str(random.randrange(100000, 999999) )
        
        #timeout
        numb = messageList[1]
        unit = messageList[2]
        
        Permissions = messageList[3::]
        
        if numb.isdigit() and unit in times.keys():       
            timeOut = int(times[unit]*float(numb)+time.time())
            print('timeOut',timeOut)
            startTasks.runLocked(access.addCode, 
                                 resourceLocks['codesData'], 
                                 codeId, Permissions, timeOut)
            sendrpl(message, "done, the key number is: "+str(codeId))
            return
    
    sendrpl(message, "Format:'/addKey N unit p1 p2 p3 etc'\n"+
                     "\t\t N: together with 'unit' gives when from now the "+
                     "code and permissions that come with it expire\n"+
                     "\t\t unit: minute(s)/hour(s)/day(s)/week(s)/month(s)\n"+
                     "\t\t p1 p2 p3 etc: space seperated list of the "+
                     "commands that this code gives acces to")
    return
    
#TODO check this function further
def deactivateKey(message, resourceLocks):
    messageList = message['text'].split(' ')
    
    if len(messageList) != 2:
        sendrpl(message, "Format: '/deactivateKey key'\n\t\tkey: the six digit"+
                         "key you want to deactivate, warning: all users who"+
                         "gained permissions using this will have those "+
                         "permissions rescinded")
        return
    if not messageList[1].isdigit() or len(messageList[1]) != 6:
        sendrpl(message, "Format: '/deactivateKey key'\n\t\tkey: the six digit"+
                         "key you want to deactivate, warning: all users who"+
                         "gained permissions using this will have those "+
                         "permissions rescinded")
        return
    
    codeId = messageList[1]

    #get nessesairy data
    codes = startTasks.runLocked(access.getCodes, 
                                 resourceLocks['codesData'])
    codeTimout = codes[codeId]['timeOut'] #save the timout before we set it to 0
    
    permData = startTasks.runLocked(access.getPermissionsData, 
                                    resourceLocks['permissionsData'])  
    
    #deactivate the code
    startTasks.runLocked(access.deactivateCode, 
                         resourceLocks['codesData'], 
                         codeId)

    #deactivate all commands given by this code for users that benefitted from that code
    for userId in codes[codeId]['usedBy']:  
    #   for all code users  
        permToRemove = []
        for permission in codes[codeId]['Permissions']:
        #   check if a permissions timeout is given by the code's expiration
        #   date if so set the timeout to 0
            if permData[userId]['permissions'][permission] == codeTimout:
                permToRemove.append(permission)
        
        startTasks.runLocked(access.changePermissions, 
                     resourceLocks['permissionsData'], 
                     userId, permToRemove, 0)    

    sendrpl(message, 'key has been deactivated and permissions rescinded')
    return

def key(message, resourceLocks):
  
    code = message['text'].split(' ')[1]
    userId = str(message['from']['id'])  
    
    if code.isdigit() and len(code) == 6:
        if config.totp.verify(int(code)):
            permission = 'changePerm'
            timeOut = time.time() + 60
            
            startTasks.runLocked(access.changePermissions, 
                                 resourceLocks['permissionsData'], userId, 
                                 permission, timeOut) 
                                             
            sendrpl(message, 'command code accepted, changing of permissions is'
                            +' allowed for the next 60 seconds')
            return
        else:
            codeId = str(code)
            codes = startTasks.runLocked(access.getCodes, 
                                         resourceLocks['codesData']) 

            #check if the key is valid
            if codeId in codes.keys():
                if codes[codeId]['timeOut'] == 0:
                    sendrpl(message, 'this key has been deactivated')
                elif codes[codeId]['timeOut'] < time.time():
                    sendrpl(message, 'this key has expired')
                else:
                    permData = startTasks.runLocked(access.getPermissionsData, 
                               resourceLocks['permissionsData'])  
                    
                    permToAdd = []               
                    for permission in codes[codeId]['Permissions']:
                    #   check if a permission should be added, (does the user
                    #   not already have it or is the timeout longer?)
                        if userId in permData.keys():
                            if permission in permData[userId]['permissions'].keys():
                                if (permData[userId]['permissions'][permission] 
                                < codes[codeId]['timeOut'] and 
                                permData[userId]['permissions'][permission] != -1):
                                    permToAdd.append(permission)
                            else:
                                permToAdd.append(permission)
                        else:
                            newUser(message, resourceLocks)#register user
                            permToAdd.append(permission)
                    
                    print('permToAdd:', permToAdd)
                    #add all the permissions in one go    
                    startTasks.runLocked(access.changePermissions, 
                                         resourceLocks['permissionsData'], 
                                         userId, permToAdd, 
                                         int(codes[codeId]['timeOut']))
                                              
                    if len(permToAdd) > 0:
                        #find out how long the codes are still valid in hr string
                        timeout = codes[codeId]['timeOut'] 
                        timeTillTimeout = timeout - int(time.time())
                        if timeTillTimeout > 0:
                            if timeTillTimeout > 3600:
                                dt = datetime.datetime.fromtimestamp(timeout)
                                validTill = ('; until: {:%Y-%m-%d %H:%M}'.format(dt))
                            elif timeTillTimeout > 60:
                                validTill = ("; for the next "+str(int(timeTillTimeout/60))+
                                             ' minutes\n')
                            else:
                                validTill = (permissions+str(funcName)+
                                            '(; for less then a minute\n')    
                        
                        #tell user about now possibilities
                        txt = ('key accepted you now have access to these commands: ')
                        if len(permToAdd) == 1:
                            txt = ('key accepted you now have access to this command: ')                            
                        
                        sendrpl(message, txt+
                                ', '.join(str(e) for e in codes[codeId]['Permissions'])
                                + validTill)
                        
                        #add user to list of users that use this code
                        startTasks.runLocked(access.addCodeUser, 
                                             resourceLocks['codesData'], 
                                             codeId, userId)
                    else:
                        sendrpl(message, 'activation key accepted however it '+
                                'does not grant you any additional access')
                return            
    sendrpl(message, 'this is not an valid key')
    return



