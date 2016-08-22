#!/usr/bin/python3

import config
import phue
b = config.b

def dimm(lamp):
    command =  {'transitiontime' : 100, 'ct' : 500, 'bri' : 0}
    b.set_light(lamp,command)
    
def dimmMax():
    command =  {'transitiontime' : 200, 'xy' : [0.5643, 0.4023], 'bri' : 0}
    b.set_light([2,3,4,5,6],command)
    
def Off():
    command =  {'transitiontime' : 0, 'on' : False}
    b.set_light([2,3,4,5,6],command)

#TODO something so this sets the last lamps preset
def On():
    command =  {'transitiontime' : 0, 'on' : True}
    b.set_light([2,3,4,5,6],command)

def SetPreset(preset):
    if preset == 'evening':
        command =  {'transitiontime' : 0, 'ct' : 400, 'bri' : 255}
    elif preset == 'night':
        command =  {'transitiontime' : 0, 'ct' : 500, 'bri' : 180}
    else:
        return
    b.set_light([2,3,4,5,6],command)


def turnOff(number):
    b.set_light(number, 'on', False)

def turnOn(number):
    b.set_light(number, 'on', True)

def status():
    lampOn = []
    for lamp in [1,2,4,5]:
        lampB = b.get_api()['lights'][str(lamp)]['state']['on']
        lampOn.append(lampB)
    return lampOn
