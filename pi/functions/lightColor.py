#!/usr/bin/python3

import math

def rgb_to_xy(R, G, B):
    #RGB between 0.0 and 1.0
    R /= 255.0
    G /= 255.0
    B /= 255.0
    
    #Gamma correction
    if R > 0.04045:
        R = ((R+0.055)/(1.0+0.055))**2.4
    else:
        R = R/12.92
    
    if G > 0.04045:
        G = ((G+0.055)/(1.0+0.055))**2.4
    else:
        G = G/12.92
    
    if B > 0.04045:
        B = ((B+0.055)/(1.0+0.055))**2.4
    else:
        B = B/12.92
    
    #Conversion
    X = R*0.664511+G*0.154324+B*0.162028
    Y = R*0.283881+G*0.668433+B*0.047685
    Z = R*0.000088+G*0.072310+B*0.986039
    
    x = X / (X + Y + Z)
    y = Y / (X + Y + Z)
    
    brightness = int(round(Y*255))    
    return (x, y, brightness)

def temp_to_rgb(temperature):    
    temperature /= 100
    
    if temperature <= 66:
        R = 255
    else:
        R = temperature - 60
        R = 329.698727446 * (R**-0.1332047592)
        if R < 0:
            R = 0
        if R > 255:
            R= 255
    
    if temperature <= 66:
        G = temperature
        G = 99.4708025861 * math.log(G) - 161.1195681661
        if G < 0:
            G = 0
        if G > 255:
            G = 255
    else:
        G = temperature - 60
        G = 288.1221695283 * (G**-0.0755148492)
        if G < 0:
            G = 0
        if G > 255:
            G = 255
    
    if temperature >= 66:
        B = 255
    else:
        if temperature <= 19:
            B = 0
        else:
            B = temperature - 10
            B = 138.5177312231 * math.log(B) - 305.0447927307
            if B < 0:
                B = 0
            if B > 255:
                B = 255    
    return (R, G, B)

def getHueParams(temperature):
#   takes color temperature in deg, returns philips hue xy paramater that would
#   get us that color
    rgb = temp_to_rgb(temperature)
    xyb = rgb_to_xy(rgb[0], rgb[1], rgb[2])
    xy = [xyb[0], xyb[1]]
    
    return xy



def getColorTemp(time):
#   takes a unix timestamp then calculates the part of the day we are in and 
#   applies that to a curve function that links it to the color temperature

    def eveningCurve(x): 
    #   this returns smoothly connected color temps suitable for the evening
    #   values found using informaticavia values and https://mycurvefit.com/
        y = -74005.37 + 2.846501*x - 0.00003635958*x**2 + 1.551974e-10*x**3
        return y

    partOfDay = int(time) % (3600*24) #how far we are this day in seconds
    
    if partOfDay >= 6*3600 and partOfDay <= 18*3600:
        temperature = 215 #day value btw 6:00 uur sochtends en 18:00
    elif partOfDay > 23*3600 or partOfDay < 6*3600:
        temperature = 500 #night value
    else:
        #for the evening take a slowely decreasing curve
        temperature = eveningCurve(partOfDay)
    return temperature



def getColorTemp2(time):
#   takes a unix timestamp then calculates the part of the day we are in and 
#   applies that to a curve function that links it to the color temperature

    def eveningCurve(x): 
    #   this returns smoothly connected color temps suitable for the evening
    #   values found using informaticavia values and https://mycurvefit.com/
        y = 362.8982 + (4541.467 - 362.8982)/(1 + (x/79134.12)**20.01336)
        return y

    partOfDay = int(time) % (3600*24) #how far we are this day in seconds
    
    if partOfDay >= 6*3600 and partOfDay <= 18*3600:
        temperature = 4500 #day value
    elif partOfDay > 23*3600 or partOfDay < 6*3600:
        temperature = 1700 #night value
    else:
        #for the evening take a slowely decreasing curve
        temperature = eveningCurve(partOfDay)
    return temperature








