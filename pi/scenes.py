#!/usr/bin/python3
#!/usr/bin/python3
################################################################################
#                                      Scenes                                  #
#                               by David Kleingeld                             #
#                                                                              #
#A scene is defined as a change to the room envirement, lighting, sound and    #
#if certain machines are on, such as the computer                              #
################################################################################
import functions.audio as audio
import functions.computer as comp
import functions.lamps as lamps
import functions.strwrc
import functions.misc as misc
import data.access as access

import config
import time
import phue
import json
import datetime
import alsaaudio
import subprocess

b = config.b

mpdConnected = functions.audio.mpdConnected
mpdIp = config.mpdIp
port = config.port


def allOff():
    lamps.Off()

def allOn():
    lamps.On()


#TODO write functino
def buildPlaylist(currentFile):
    print('wip') 
    #using current file string get next file and store
    #also store resumePoint and end location
    #loop storing current position and checking if not too close to end
    #

def audiobook():    
    EbookStatus = audio.ebookInfo()
    data = json.loads(open('/home/pi/status.json').read())
    if EbookStatus == False:
        misc.httpReplyQueue.put('starting')
        with mpdConnected(mpdIp, port) as mpd:
            audio.clearQueue(mpd)
            
            mpd.add(data['lastFile'])
            mpd.seek(0, float(data['resumePoint'])-config.deltaResume)
            
            #setVolume(volmod=float(-20))
            mpd.pause(0) #these two commands needed to start playback
            
            #TODO build up playlist
            #buildPlaylist(data['lastFile'])
            
    else:
        misc.httpReplyQueue.put('stopping')
        with mpdConnected(mpdIp, port) as mpd:
            currentItem = mpd.playlistinfo(int(EbookStatus['song']))[0]

            lastFile = currentItem['file']
            resumePoint = EbookStatus['elapsed']
            
            if isinstance(data, dict):
            #   add/update the keys to the existing dict
                data['lastFile'] = lastFile
                data['resumePoint'] = resumePoint
            else: 
            #   make the dictionary with keys
                data = {'lastFile' : lastFile, 
                        'resumePoint' : resumePoint} 
            
            with open('/home/pi/status.json', 'w') as outfile:
                json.dump(data, outfile)

            #stop playing            
            mpd.stop()



def goingToSleep():
#    turnOffPc()

    #check if we are not listening to an ebook already
    if audio.ebookInfo() == False:
        with mpdConnected(mpdIp, port) as mpd:        
            audio.clearQueue(mpd)
            audio.addFromPlayList(mpd, 'sleep', time=500)  
            mpd.play(0)  #these two commands needed to start playback
            mpd.pause(0) #these two commands needed to start playback
        
    for lamp in [1,2,3,4,5,6]:
        lampB = b.get_api()['lights'][str(lamp)]['state']['bri']
        if lampB > 120:
            lamps.dimm(lamp)
    
    sleep(10)
    lamps.dimmMax()
    
    sleep(20)        
    lamps.Off() 
    audio.volumeFade(config.sleepVolume, 5)



def wakeup():
    
    def playMusic(weekday):
        with mpdConnected(config.mpdIp, config.port) as mpd:       
            audio.clearQueue(mpd)
            
            if weekday:
                audio.addFromPlayList(mpd, 'calm', time=300)
                audio.addFromPlayList(mpd, 'energetic', numb=3)
            else:
                audio.addFromPlayList(mpd, 'calm', time=300)
                audio.addFromPlayList(mpd, 'jazz', time=300)
            
            m = alsaaudio.Mixer()                 
            m.setvolume(10)
            mpd.play(0)  #these two commands needed to start playback
            mpd.pause(0) #these two commands needed to start playback
            audio.volumeFade(config.wakeupVolume, 5)
        return    
    
    Time_ = int(config.wakeupPeriod*600)
    #fade to max brightness with current color 
    b.set_light([2,3,4,5,6], {'on' : True, 'bri': 0, 'ct': 500})
    command =  {'transitiontime' : Time_, 'on' : True, 'bri' : 255, 'ct' : 198}
    b.set_light([2,3,4,5,6], command)
    
    time.sleep(config.wakeupPeriod*60/2)
    
    #check if weekday
    d = datetime.datetime.now()
    weekday = True if d.isoweekday() in range(1, 6) else False
   
    status, songId = startTasks.runLocked(play, resourceLocks['mpd'], streamingLink)   








def movie():
    #check if main pc is on, if not start it and wait till its on 
    command = ["/home/pi/bin/wakeMainPc.sh"]
            
    popen = subprocess.Popen(command, stdout=subprocess.PIPE)
    popen.communicate() #wait till its completed


    #open an ssh connection to my main computer
    #start kodi
    command = ["ssh", "-o ConnectTimeout=5", 
              "kleingeld@192.168.1.12",
              "if pgrep 'kodi' > /dev/null; "
                    "then echo "
                    "'Running already not starting new instance'; "
                    "else export DISPLAY=:0; "
                    "kodi -fs & "
              "fi; "                     
              "echo active > /dev/input/ckb1/cmd;"
              "echo rgb 000000 > /dev/input/ckb1/cmd;"
              "exit"]
              
    popen = subprocess.Popen(command, stdout=subprocess.PIPE)


    #FADE LIGHTS ON
    color= [0.5339, 0.394]

    #set bureau lamp transistion time 20 seconds
    b.set_light([3], {'on' : True, 'bri': 40, 'xy': color, 'transitiontime' : 200})
    b.set_light([1], {'on' : False, 'transitiontime' : 200, 'xy': color})
    b.set_light([2], {'on' : True, 'bri': 0, 'xy': color, 'transitiontime' : 200})

def jazz():
    with mpdConnected(mpdIp, port) as mpd:
        
        volMod=2
        audio.clearQueue(mpd)
        mpd.setvol(int((100-20)/volMod))
        mpd.load('jazz')
        mpd.play(0)  #these two commands needed to start playback
        mpd.pause(0) #these two commands needed to start playback


def testmpd():
    with mpdConnected(mpdIp, port) as mpd:
        while True:
            volMod=2
            
            audio.clearQueue(mpd)
            mpd.setvol(int((100-20)/volMod))
            mpd.load('jazz')
            mpd.play(0)  #these two commands needed to start playback
            mpd.pause(0) #these two commands needed to start playback

def lockdown():
    #run the system lockdown script and let it take care of things 
    command1 = ["/home/pi/bin/lockdownPc.sh"]
    #command2 = ["/home/pi/bin/lockdownWan.sh"]
    
    popen1 = subprocess.Popen(command1, stdout=subprocess.PIPE)
    #popen2 = subprocess.Popen(command2, stdout=subprocess.PIPE)
    print('waiting')    
    popen1.communicate() #wait till its completed
    #popen2.communicate() #wait till its completed
    
    #pulse lamp color to show lockdown completed
    #getCurrentSettings
    isOn    = b.get_api()['lights']['3']['state']['on']
    xyColor = b.get_api()['lights']['3']['state']['xy']
    bri     = b.get_api()['lights']['3']['state']['bri']

    print('gottinfo')
    #pulse
    pulsBri = bri
    if isOn == False:
        pulsBri = 100
    #integer here since this is a list
    b.set_light([3], {'on' : True, 'bri': pulsBri, 'xy': [0.1684, 0.0416], 'transitiontime' : 0})
    print('setlamp1')    
    #reset settings
    sleep(2)
    b.set_light([3], {'on' : isOn, 'bri': bri, 'xy': xyColor, 'transitiontime' : 10})
    print('setlamp2') 

def leftHome():
    b.set_light(3,'on', False)
    audio.musicOff()

def returnedHome():
    playlist = 'return'
    time = strftime("%H")

    #time dep. playlist selection
    if int(time) > 17:
        playlist = 'return calm'

    b.set_light(3,'on', True) 
          
    with mpdConnected(mpdIp, port) as mpd:
        clearQueue(mpd)
        addFromPlayList(mpd, playlist, time=600) #turn on music with welcome back playlist
        setVolume(volmod=float(-20))
        mpd.play(0)  #these two commands needed to start playback
        mpd.pause(0) #these two commands needed to start playback

def SetAlarm(raw):  
    minTillAlarm= raw[10::]
    minTillWake= int(float(minTillAlarm)-wakeupPeriod)

    subprocess.call('at now + '+str(minTillWake)+
                    ' minutes <<< "GET 192.168.1.10:8080/Scene/wakeup"'
                    ,shell=True, executable='/bin/bash')
