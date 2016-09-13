#!/usr/bin/python3

import alsaaudio
import random
import musicpd
import time

import config


class mpdConnected():
# class used to connect to the mpd server with a 'with' statment. This makes
# sure we always disconnect nicely
    def __init__(self, mpdIp, port):
        self.mpdIp = mpdIp
        self.port = port 
        self.client = musicpd.MPDClient()       
    def __enter__(self):
        self.client.connect(mpdIp, port)
        return self.client  
    def __exit__(self, type, value, traceback):
        self.client.disconnect()

mpdIp = config.mpdIp
port = config.port

def playStream(streamingLink):
    with mpdConnected(config.mpdIp, config.port) as mpd:                 
        #store info about current mpd status
        status = mpd.status()         
        songId = mpd.addid(streamingLink)
        #TODO set volume?

        mpd.playid(songId)  #these two commands needed to start playback
        mpd.pause(0) #these two commands needed to start playback

    return status, songId

def restorePrev(status, songId):
    with mpdConnected(config.mpdIp, config.port) as mpd:   
        #restore playback conditions
        mpd.deleteid(songId)
        if status['state'] == 'pause':
            mpd.pause(1)
        elif status['state'] == 'stop':
            mpd.stop()
        else:
            mpd.seek(0, float(status['elapsed'])-1)
    return

def volume():
    m = alsaaudio.Mixer() 
    currentVolume = str( m.getvolume()[0])
    return currentVolume
    
def musicStatus():
    with mpdConnected(config.mpdIp, config.port) as mpd:
        status = mpd.status()
        if status['state'] == 'stop':
            return 'nothing playing'
        else:
            currentItem = mpd.playlistinfo(int(status['song']))[0]
            
            #assume title from file path
            Title = currentItem['file']
            Title = Title[Title.find('/')+1:Title.find('.')]
            Title = Title.replace("_", " ")
            
            artist = '' #get artist info if availible
            if 'artist' in currentItem:
                artist = currentItem['artist']+' - '
            return str(' '+artist+Title)

def ebookInfo():
#   check if an ebook is playing, return false if not else return the ebook info
    with mpdConnected(mpdIp, port) as mpd:
        status = mpd.status()
        if status['state'] == 'stop':
            return False
        else:
            currentItem = mpd.playlistinfo(int(status['song']))[0]
            if 'Ebooks' in currentItem['file']:
                return status
            else:
                return False
            
def clearQueue(mpd):
#   delete from begin till end of queue
    mpd.command_list_ok_begin()       # start a command list
    mpd.status()
    raw = mpd.command_list_end()
    length = raw[0]['playlistlength']
    
    if int(length) > 0:
        mpd.delete((0, ))
        
wasPauzed = False #make sure this is rememberd outside the loop   
def Playback(raw):
    m = alsaaudio.Mixer()
    global wasPauzed # Needed to modify global copy of globvar
    todo= raw[12::]
    with mpdConnected(config.mpdIp, config.port) as mpd:   
        #if signal comes from auto pause if playing something on main
        #computer bash script (sends: Playback:A:pause )
        if raw[10:11] == 'A':  
                if todo == 'pause' and mpd.status()['state'] == 'play':
                    mpd.pause(1)
                    wasPauzed = True
                elif todo == 'resume' and wasPauzed:
                    Volume = m.getvolume()[0]  
                    m.setvolume(int(Volume/2))
                    mpd.pause(0)
                    volumeFade(int(Volume/2 + Volume/10), 0.001)
                    volumeFade(int(Volume/2 + Volume/5), 0.05)                         
                    volumeFade(int(Volume), 0.1)                 
                    wasPauzed = False
        #else pause/play without any checking (for keyboard launched events)                   
        else:
            if mpd.status()['state'] == 'play':
                mpd.pause(1)
            else:
                mpd.pause(0) 
                
def volumeFade(targetVol, speed):
    m = alsaaudio.Mixer()
    currentVolume = m.getvolume()[0]   
   
    if targetVol < currentVolume :
        fade = range(targetVol, currentVolume)
        fade = reversed(fade)
    else:
        fade = range(currentVolume, targetVol)
    for volume in fade:
        m.setvolume(volume)
        time.sleep(speed)

def volumeUp():
    m = alsaaudio.Mixer()
    currentVolume = m.getvolume()[0]
    newVol = int(currentVolume)+config.volInterval
    if newVol > 100:
        newVol = 100
    m.setvolume(int(newVol))

def volumeDown():
    m = alsaaudio.Mixer()
    currentVolume = m.getvolume()[0]
    newVol = int(currentVolume)-config.volInterval
    if newVol < 0:
        newVol = 0
    m.setvolume(int(newVol))

def setVolume(**kwargs): 
    m = alsaaudio.Mixer()
    volmod = int(kwargs.get('volmod', 0))
    
    with open('/home/pi/bin/status.dat', 'r') as f:
        status = json.load(f)

    if status == 'On':
        volumeFade(100, 0.1)
    elif status == 'Off':
        m.setvolume(int(60+volmod))

def musicOff():
    with mpdConnected(config.mpdIp, config.port) as mpd:
        mpd.pause(1)

def addFromPlayList(mpd, playlist, **kwargs):
#   will fail if the current playlist contains duplicates

    numb        = kwargs.get('numb', None)
    time_       = kwargs.get('time', None)
    variance    = kwargs.get('time', 25)
    
    #print(numb)
    pList = mpd.listplaylistinfo(playlist)

    #remove already queud songs from the list we choose form
    #remove songs that are too long  
    for i in reversed(range(len(pList))): #reversed so we can keep iterating after del
        for qSong in mpd.playlistinfo():
            if pList[i]['file'] == qSong['file']:
                del pList[i]
                break #break saves time and prevents missing item 
            elif time_:#needed to allow not passing time
                if int(pList[i]['time']) > int(time_ - variance):
                    del pList[i]
                    break              

    if numb:        
        #check if that list still has length
        for i in range(numb):
            l = len(pList)
            if l > 0:
                toadd = random.randint(0, l-1)        
                mpd.add(pList[toadd]['file'])
                del pList[toadd]
                
            else:
                print('list is empty')
    
    if time_:        
        #time based
        #calculate max length of playlist to make sure an exit condition exists
        totalTime = 0
        for i in range(len(pList)):
            totalTime += int(pList[i]['time'])
        if totalTime < time_:
            print('totalTime: '+str(totalTime))
            print('not enough songs in playlist')
            return
        else:
            totalTime = 0
            while totalTime < time_:
                toadd = random.randint(0, len(pList)-1)        
                mpd.add(pList[toadd]['file'])
                
                for i in range(len(pList)):
                    totalTime += int(pList[toadd]['time'])   
                                 
                del pList[toadd]
