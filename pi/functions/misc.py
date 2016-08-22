#!/usr/bin/python3
import subprocess
import multiprocessing
import os
import time 

import config

httpReplyQueue = multiprocessing.Queue(maxsize=0)

def systemStatus():
#   is thread safe gets the load average of the device the script runs on
    load = list(os.getloadavg())
    return load
    

