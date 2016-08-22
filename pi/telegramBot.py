#!/usr/bin/python3
################################################################################
#                               Home Pi Responder                              #
#                               by David Kleingeld                             #
#                                                                              #
#This program responds to Http requests and facilitates communication between. #
#Smartphones running the tasker app and a linux home server                    #
################################################################################
#private libs

import requests
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
import ssl
import json
import cgi #needed to parse multiform data
import threading
import datetime
from config import bcolors

import functions.bot as bot
import startTasks
import config
import data.access as access

def enableWebhook():
#   enable the webhook and upload the certificate 
    params = {'url': 'https://deviousd.duckdns.org:8443/'}
    r = requests.get("https://api.telegram.org/bot"+config.token+"/setWebhook", 
                      params=params,
                      files={'certificate' : open('/home/pi/bin/homeAutomation/data/PUBLIC.pem', 'r')})
    print("server replies:",r.json())

def messageInfo(message):
    from_name = message['from']['first_name']
    
    if 'title' in message['chat']:
        chat_name = message['chat']['title']
    elif 'username' in message['chat']:
        chat_name = "@"+message['chat']['username']
    else:
        chat_name = "?"
    
    if 'text' in message.keys():
        text = message['text']
    else:
        text = bcolors.OKGREEN+'no text was given'+bcolors.ENDC
    
    print("<"+chat_name+"> "+from_name+": "+text, end='\n')

def debugMessage(result):
    messageInfo(result['message'])
    print('recieved message send to telegram on/at: ',end="")
    print(bcolors.OKGREEN+(datetime.datetime.fromtimestamp(int(result['message']['date'])
                          ).strftime('%Y-%m-%d %H:%M:%S'))
                          +bcolors.ENDC)

    print('full json dict:\n'+str(result))





"""
Respond functions, these start other functions, similar to the HTTP responder
service exept these already run in theire own thread. They start functions using
the Queue system
"""

def handleText(message, tasksQueue, resourceLocks, sensorGet, sensorGetBack, 
               sensorRequest, analysisRq, lightSceneQueue):   

    if message['text'] == '/help':
        bot.help(message, resourceLocks)

    elif '/key' in message['text']:
        bot.key(message, resourceLocks)

    if message['text'].split(' ')[0] == '/stream':
        if bot.checkPerm(message, resourceLocks, 'stream'):
            bot.streamAudio(message, resourceLocks, False)    
    
    if (message['text'][:23] == 'https://www.youtube.com' or 
       message['text'][:13] == 'https://youtu'):
        if bot.checkPerm(message, resourceLocks, 'stream'):
            bot.streamAudio(message, resourceLocks, True)

    if message['text'] == '/status':
        if bot.checkPerm(message, resourceLocks, 'status'):
            bot.status(message, resourceLocks, sensorGet, sensorGetBack, sensorRequest)

    elif message['text'] == '/strwrc':
        if bot.checkPerm(message, resourceLocks, 'strwrc'):
            bot.strwrc(message, resourceLocks)

    elif message['text'][:6] == '/graph':
        if bot.checkPerm(message, resourceLocks, 'graph'):
            bot.graph(message, resourceLocks, analysisRq)
    
    elif message['text'] == '/lamps':
        if bot.checkPerm(message, resourceLocks, 'lamps'):
            bot.lamps_1(message, tasksQueue, resourceLocks, lightSceneQueue)  

    elif '/showPerm' in message['text']:
        if bot.checkPerm(message, resourceLocks, 'showPerm'):
            bot.showPerm(message, resourceLocks)

    elif '/changePerm' in message['text']:
        if bot.checkPerm(message, resourceLocks, 'changePerm'):
            bot.changePerm(message, resourceLocks)

    elif '/addKey' in message['text']:
        if bot.checkPerm(message, resourceLocks, 'addKey'):
            bot.addKey(message, resourceLocks)

    elif '/deactivateKey' in message['text']:
        if bot.checkPerm(message, resourceLocks, 'deactivateKey'):
            bot.deactivateKey(message, resourceLocks)

    elif message['text'] == '/wakeup':
        if bot.checkPerm(message, resourceLocks, 'wakeup'):
            bot.wakeUp_1(message, resourceLocks, tasksQueue)

    elif message['text'] == '/broadcast':
        if bot.checkPerm(message, resourceLocks, 'broadcast'):
            bot.broadcast_1(message, tasksQueue, resourceLocks)  


def handleReply(message, resourceLocks):
    #TODO user security control
    replyData = startTasks.runLocked(access.getReplyData, 
                                     resourceLocks['replyData'])
    userId = message['from']['id']
    if userId in replyData.keys():
        replyToId = message['reply_to_message']['message_id']
        if replyToId in replyData[userId]:
            func = replyData[userId][replyToId]['function']
            args = replyData[userId][replyToId]['funcArgs']
            if not args:
                func(message)
            else:
                func(message, *args)
                
    #if not in replydata do not continue
    else:
        return
    #start the task in replydata
    startTasks.runLocked(access.deleteReplyData, resourceLocks['replyData'],
                         userId, replyToId)


#def handleNonTextReply(message, authLevels, replyList):









def process_message(result, tasksQueue, resourceLocks, sensorGet, sensorGetBack,
                    sensorRequest, analysisRq, lightSceneQueue):
    message = result['message']
    debugMessage(result)
    if 'reply_to_message' in message:
        if message['reply_to_message']['from']['username'] == 'DeviousBot':
            handleReply(message, resourceLocks)
    elif 'text' in message:
        if config.botName in message['text']:
            message['text'] = message['text'][:-len(config.botName)]
        handleText(message, tasksQueue, resourceLocks, sensorGet,
                   sensorGetBack, sensorRequest, analysisRq, lightSceneQueue)
    return

def genHttpClass(tasksQueue, resourceLocks, sensorGet, sensorGetBack,
                 sensorRequest, analysisRq, lightSceneQueue):
#   used to pass above vars to myhandler class in a way that works..... je zet
#   eigl de vars in de scope van de class en daarom werky, soort constructor
    class MyHandler(BaseHTTPRequestHandler):
    #   check http get requests and start the corresponding functions
        def do_POST(self):
            #reply data recieved succesfully (otherwise endless spam)
            message = json.dumps({})
            self.send_response(200)
            self.send_header('Content-type','application/json')
            self.end_headers()
            self.wfile.write(message.encode('utf-8')) #send bytestring not utf8  
            
            #decode and read the data
            content_len = int(self.headers['content-length'])
            post_body = self.rfile.read(content_len)
            post_body_str = post_body.decode("utf-8")
            data = json.loads(post_body_str)
            
            #process the message in a new thread
            t = threading.Thread(target= process_message, args = (data, 
                                 tasksQueue, resourceLocks, sensorGet,
                                 sensorGetBack, sensorRequest, analysisRq,
                                 lightSceneQueue))
            t.start()
            return
    return MyHandler
    
def HttpRecieveServer(tasksQueue, resourceLocks, sensorGet, sensorGetBack,
                      sensorRequest, analysisRq, lightSceneQueue):
    botServer = HTTPServer(("192.168.1.10", 8443), 
                            genHttpClass(tasksQueue, resourceLocks, sensorGet,
                                         sensorGetBack, sensorRequest,
                                         analysisRq, lightSceneQueue))
    botServer.socket = ssl.wrap_socket(botServer.socket, 
                       certfile='/home/pi/bin/homeAutomation/data/PUBLIC.pem',
                       keyfile='/home/pi/bin/homeAutomation/data/PRIVATE.key',
                       server_side=True)
    try:
        print("starting botServer")
        botServer.serve_forever()
    except KeyboardInterrupt:
        pass
    botServer.server_close()

#enableWebhook() #TODO sometimes fixes shit?
