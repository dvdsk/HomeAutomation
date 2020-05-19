use telegram_bot::types::refs::ChatId;
use crate::input::web_api::server::State;
use crate::input::alarms::Alarm;
use crate::controller::Event;
use super::send_text;
use chrono::{TimeZone};
use chrono_tz::Europe::Amsterdam;
use chrono::{Local, Utc, DateTime};
use chrono::Timelike;
use chrono::Duration;
use hex::FromHex;

#[derive(Debug)]
pub enum Error {
    NoArguments,
    InvalidOption,
    CouldNotRespond,
    NoTimeArgument,
    InvalidHourArgument,
    InvalidMinuteArgument,
    WrongTimeFormat,
    NoAlarmId,
    MalformedAlarmId,
    CouldNotRemove,
}

pub async fn handle<'a>(chat_id: ChatId, token: &str, mut args: std::str::SplitWhitespace<'a>, state: &State)
     -> Result<(),Error> {

    let command = args.next().ok_or(Error::NoArguments)?;
    dbg!(command);

    match command {
        "list" => {
            list_alarm(chat_id, token, state).await?
        }
        "add" => {
            add_alarm(chat_id, token, args, state).await?
        }
        "remove" => {
            remove_alarm(chat_id, token, args, state).await?
        }
        &_ => Err(Error::InvalidOption)?
    }

    Ok(())
}

pub async fn add_alarm<'a>(chat_id: ChatId, token: &str, mut args: std::str::SplitWhitespace<'a>, state: &State)
    -> Result<(),Error> {
    let time = args.next().ok_or(Error::NoTimeArgument)?;

    //assumes hh:mm
    let mut time = time.split(':');
    let hour = time.next()
        .ok_or(Error::WrongTimeFormat)?
        .parse::<u32>()
        .map_err(|e| Error::InvalidHourArgument)?;
    let min = time.next()
        .ok_or(Error::WrongTimeFormat)?
        .parse::<u32>()
        .map_err(|e| Error::InvalidHourArgument)?;
    
    if hour > 24 {return Err(Error::InvalidHourArgument);}
    if min > 60 {return Err(Error::InvalidMinuteArgument);}

    //let now = Amsterdam::now();
    let now = Local::now();
    //let now = Amsterdam.ymd(2016, 5, 10).and_hms(12, 0, 0);
    //let now = dt.with_timezone

    let alarm_sec_after_midnight = hour*3600+min*60;
    let till_alarm = if alarm_sec_after_midnight < now.num_seconds_from_midnight(){
        let seconds_left_in_day = 3600*24 - now.num_seconds_from_midnight();
        Duration::seconds(seconds_left_in_day as i64)
            +Duration::seconds(alarm_sec_after_midnight as i64)
    } else {
        let seconds_till_alarm = alarm_sec_after_midnight 
            - now.num_seconds_from_midnight();
        Duration::seconds(seconds_till_alarm as i64)        
    };


    let alarm = Alarm::from(Utc::now()+till_alarm, 
        Event::Alarm, 
        Some(std::time::Duration::from_secs(3600*2)));

    state.alarms.add_alarm(alarm).await.unwrap();
    
    let text = format!("set alarm for over {} hours and {} minutes from now",
        till_alarm.num_hours(), till_alarm.num_minutes() % 60);

    send_text(chat_id, token, text).await
        .map_err(|_| Error::CouldNotRespond)?;

    Ok(())
}

pub async fn list_alarm(chat_id: ChatId, token: &str, state: &State)
    -> Result<(), Error> {
    
    let alarms = state.alarms.list();
    let mut list = String::with_capacity(alarms.len()*100);
    
    for (id, alarm) in alarms.into_iter() {
        
        list.push_str(&format!("{:x}, {}, {:?}, {:?}",
            id,
            &alarm.time.with_timezone(&Local).to_rfc2822(),
            &alarm.action,
            &alarm.expiration,
        ));
        list.push_str("\n");
    }
    if list.is_empty() {
        list.push_str("no alarms set");
    }

    send_text(chat_id, token, list).await//.unwrap();
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}

pub async fn remove_alarm<'a>(chat_id: ChatId, token: &str, mut args: std::str::SplitWhitespace<'a>, state: &State)
    -> Result<(), Error> {
    
    let to_remove = dbg!(args.next())
        .ok_or(Error::NoAlarmId)?;
    let to_remove = u64::from_str_radix(to_remove, 16).unwrap();
    
    let removed = state.alarms.remove_alarm(to_remove)
        .unwrap(); //TODO //FIXME
        //.map_err(|_| Error::CouldNotRemove)?;

    let text = if let Some(alarm) = removed {
        format!("removed alarm {:x}: {}, {:?}, {:?}",
            to_remove,
            &alarm.time.with_timezone(&Local).to_rfc2822(),
            &alarm.action,
            &alarm.expiration,
        )
    } else {
        String::from("Alarm does not exist, no alarm removed")
    };

    send_text(chat_id, token, text).await//.unwrap();
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}