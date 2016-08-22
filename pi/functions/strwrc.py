#!/usr/bin/python3
################################################################################
#                         STRW resource checker (strwrc)                       #
#                               by David Kleingeld                             # 
#                                 november 2015                                #
#                                                                              #
#This program presents a list of least used computation servers availible.     #
#Requirments:                                                                  #
#   -python3                                                                   #
#   -numpy for python3                                                         #
#Installation:                                                                 #
#   -place this file in the bin folder in your home folder (~/bin) you can     #
#    create it if it is not yet there. Now the command strwrc is availible     #
#    from anywhere through the terminal.                                       #
################################################################################

#todo, make the ssh key copy get the right key, debug the ssh copy errors
#comment more shit


import time

import subprocess
import threading
import numpy as np
from multiprocessing import Process, Queue, Pool, Pipe
import os.path


#advantage of multiprocessing over threading: multiple cpu cores possible
#disadvantage: larger memeory footprint

################################################################################
#                             Script setup                                     #
################################################################################

#list of pc numbers availible
tocheck = np.arange(0,22)
#timeout on the ssh connection
timeout = 5
number = 0
strwHostename = 'kleingeld'


################################################################################

def ssh(number, timeout):
#   open an ssh connection to a strw computer and run the uptime command to
#   get the current number of users and load avarages
    
    command =["ssh", "-o ConnectTimeout="+str(timeout),"-i/home/pi/.ssh/strwrc", 
              (strwHostename+"@pczaal"+str(number)+".strw.leidenuniv.nl"), "uptime"]
    
    popen = subprocess.Popen(command, stdout=subprocess.PIPE)
    result = popen.stdout.readlines()
    return result
    
def only_numerics(seq):
#   removes all non digets from a string
    
    seq_type= type(seq)
    return seq_type().join(filter(seq_type.isdigit, seq))

def store(number, output):
#   take the raw output from the ssh function and reorder it into a list with
#   the number of the computer and then the load avarages

    #casting to string needed cause output is a bytes type
    raw = str(output[0]).split(',')
    #compensates for missing number of days (happens if reboot pc < -24 hours)
    dayCompensatonHack = 1-str(output[0]).count('day')

    numbOfUsers = float(only_numerics(raw[2-dayCompensatonHack]))
    load_1m  = float(raw[3-dayCompensatonHack][16::])
    load_5m  = float(raw[4-dayCompensatonHack][1::])
    load_15m = float(raw[5-dayCompensatonHack][1:5])
    
    row = [number, load_1m, load_5m, load_15m, numbOfUsers]
    return(row)
    
def sort(results):
#   sort the results on load avarages with the lowest load on top      
    results = np.asarray(results)
    
    #try to minimize this value for sorting
    sortQualifiers = results[:,1]+results[:,2]+results[:,3]
    indices = np.argsort(sortQualifiers)
    return results[indices,:]

def noErrors(output):
#   check if the ssh output is something else then expected   
    if not output:
        return False
    else:
        raw = output[0]
        if raw[10:12] == b'up':
            return True
        else:
            return False
    
def printSortedTable(res):
#   print the sorted results in a table  
    a = ('host      | users |      load \n'
          '                            1m,   5m,    15m\n')
    for R in res:
        spatie = ' '
        if R[0] < 10:
            spatie = '   ' 
        a = a+(('pczaal{:.0f} {}  |{:.0f}| {:.2f}, {:.2f},  {:.2f}\n')
               .format(R[0], spatie, R[4], R[1], R[2], R[3]))
    numbOfNonResp = (len(tocheck)-len(res))
    hosts = 'hosts'
    were = 'were'
    if numbOfNonResp == 1:
        hosts = 'host'
        were = 'was'
    a = a+('\n'+str(numbOfNonResp)+" "+str(hosts)+" "+were+" unresponsive\n")
    return a

def removeNone(lis):
#   remove all None items from a list
    lis2 = []
    for idx, val in enumerate(lis):
        if val is not None:
            lis2.append(val)
    return lis2


def start(number):
#   start the ssh connection check the output and store it nicely
    output = ssh(number, timeout)
    if noErrors(output):
        row = store(number, output)
        return row




def checkStrwLoad():
    results = []
        
    #run the ssh connections and storing in parralel
    with Pool(processes=len(tocheck)) as pool:
        results = pool.map(start, tocheck)
        results = removeNone(results)
    
    
    #sort and print the results
    if len(results) is not 0:
        results_sorted = sort(results)
    return printSortedTable(results_sorted)



