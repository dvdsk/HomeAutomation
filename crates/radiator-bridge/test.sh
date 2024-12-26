#!/usr/bin/env bash

# Print if argument matches regex:
#
# you can do this with: RUST_LOG='[{topic=.*small_bedroom:piano.*}]=trace,info'
# that will print every log at trace level or higher that is inside an
# instrumented function with an argument topic for which the regex
# .*small_bedroom.* evaluates as true
#
#
# Print if in function:
# RUST_LOG='[function_name]=trace'
        
# RUST_LOG='[parse_message]=trace,[{device_name=.*:radi.*}]=trace,[{friendly_name=.*:radi.*}]=trace,info' 

RUST_LOG=data_server=trace cargo r -- \
	--data-server-subscribe=192.168.1.43:1235 \
	--data-server-update=192.168.1.43:1234 \
	--mqtt-ip=192.168.1.43
