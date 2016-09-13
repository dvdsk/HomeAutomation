#!/usr/bin/python3
################################################################################
#                               Home Pi Responder                              #
#                               by David Kleingeld                             #
#                                                                              #
#This program responds to Http requests and facilitates communication between. #
#Smartphones running the tasker app and a linux home server                    #
################################################################################
import pyotp
from phue import Bridge
import alsaaudio

#TODO
#make volume fade always parallel
#make setvolume functional

##########################
class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

##########################


################################################################################
#SETUP
################################################################################

httpPort	    = 8080
mpdIp           = '192.168.1.10'
port            = 6600
b               = Bridge('192.168.1.11')
m               = alsaaudio.Mixer('Master')
totp            = pyotp.TOTP('CT5IB3C3CMPLJ2XV') #also installed on watch
sleepVolume     = 5                        #in percent 
wakeupVolume    = 40

wakeupPeriod    = 5                         #in minutes
deltaResume     = 5                 #how much earlier should we resume from a pauzed point? in seoncds

'''wakeup playlist time'''
time_           = 300                       #in seconds
variance        = 25                        #in seconds
volInterval     = 5                         #in percent

'''google api key'''
googleKey = 'AIzaSyDDcwSb03yZgSOHKMy-6PNnWbAXmlh1gO0'

'''bot settings'''
token = '193483364:AAHji5yfbb_DHjaZl43U_KynVHa3fFWdvh4'
botName = '@DeviousBot'

################################################################################
#STRWRC settings
################################################################################
minDeltaT_strwRC    = 900 
timeout_strwRC      = 5 #requirs restart

################################################################################
#BOT RESPONSES
################################################################################


accesDenied = ["ACCESS RESTRICTED\nsecurity level "," or above is required to access "]#+something

commandList = ['/help - get a list of commands \n/override - temporarily evaluate clearance level',

               '/strwrc - get sterrenwacht hosts load',
               
               '/status - get system status',
               
               '/lamps - control lighting system', 
               
               '', 
               
               '', 
               
               '/broadcast - play a recording on speakers', 
               
               '', 
               
               '/wakeup - start morning wakeup', 
               
               '/clearance - adjust security clearances']

classified = "That information is classified"
verified = "Command Codes verified"
deactivation = "Deactivation complete, now disconnected from authorised session"

unknownCommand = "Your request does not fall within current guidlines"
Deactivation = "Please confirm Bot deactivation request"

InGroup = "Inquries into command functions are not accepted from your present location"
logs = "logs accesed"

publicGroup = "Please confirm ouput over unauthorised session" 

