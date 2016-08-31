#!/usr/bin/python3

#general libs
from http.server import BaseHTTPRequestHandler
from socketserver import TCPServer
import threading

#private libs
import config
from startTasks import tasksQueue
from status import lightSceneQueue, sleeping
import scenes
import status


import functions.audio as audio
import functions.computer as comp
import functions.lamps as lamps
import functions.strwrc 
import functions.misc as misc

def awnserClient(httpServer, message):
    httpServer.send_response(200)
    httpServer.send_header('Content-type','text')
    httpServer.end_headers()
    httpServer.wfile.write(message.encode('utf-8')) #send bytestring not utf8     

        
class MyHandler(BaseHTTPRequestHandler):
#   check http get requests and start the corresponding functions
    def do_GET(self):
        if 'SetAlarm' in self.path:
            tasksQueue.put([scenes.SetAlarm, self.path])
       
        elif 'audiobook' in self.path:    
            tasksQueue.put([scenes.audiobook, None])  
            message = misc.httpReplyQueue.get(timeout=0.1)
            awnserClient(self, message) 
            
        elif 'Scene' in self.path:
            sceneName= self.path[7::]
            if sceneName == 'evening':
                lightSceneQueue.put(status.evening)
            elif sceneName == 'night':
                tasksQueue.put([comp.startRedshift, None])
                lightSceneQueue.put(status.night)
            elif sceneName == 'normal':
                lightSceneQueue.put(status.colorLoopScene)
            elif sceneName == 'bedlight':
                lightSceneQueue.put(status.bedlight)
            elif sceneName == 'allOff':
                tasksQueue.put([scenes.allOff, None])
            elif sceneName == 'allOn':
                tasksQueue.put([scenes.allOn, None])
            elif sceneName == 'sleep':
#                print('sleeeping')
#                tasksQueue.put([scenes.goingToSleep, None])#TODO NEEDS DEBUGGING
                sleeping.set()
#                print(sleeping.is_set())
            elif sceneName == 'nosleep':
#                print('sleeeping')
#                tasksQueue.put([scenes.goingToSleep, None])#TODO NEEDS DEBUGGING
                sleeping.clear()
#                print(sleeping.is_set())
            elif sceneName == 'jazz':
                tasksQueue.put([scenes.jazz, None])
            elif sceneName == 'movie':
                tasksQueue.put([scenes.movie, None])
            elif sceneName == 'lockdown':
                tasksQueue.put([scenes.lockdown, None])
            elif sceneName == 'wakeup':
                tasksQueue.put([scenes.wakeup, None])
                lightSceneQueue.put(status.wakeup) 

        elif 'pcOff' in self.path:           
            tasksQueue.put([comp.turnOffPc, None])

        elif 'Playback' in self.path:   
            tasksQueue.put([audio.Playback, self.path])

        elif 'volumeUp' in self.path:
            tasksQueue.put([audio.volumeUp, None])

        elif 'volumeDown' in self.path:
            tasksQueue.put([audio.volumeDown, None])
            
        self.send_response(200)


        

def httpResponder():  
    print('starting http responding service')
    try:
        httpd = TCPServer(("", config.httpPort), MyHandler)
        httpd.serve_forever()
    except KeyboardInterrupt:
        httpd.shutdown() #TODO improve with code from bot.py in dev/boooottt

