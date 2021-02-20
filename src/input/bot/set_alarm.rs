use super::send_text;
use crate::input::web_api::server::State;
use chrono::{Local, Timelike};
use telegram_bot::types::refs::ChatId;

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
}

pub async fn handle<'a>(
    chat_id: ChatId,
    token: &str,
    mut args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let command = args.next().ok_or(Error::NoArguments)?;
    dbg!(command);

    match command {
        "list" => list_alarm(chat_id, token, state).await?,
        "tomorrow" => add_tomorrow(chat_id, token, args, state).await?,
        "usually" => add_usually(chat_id, token, args, state).await?,
        "remove" => remove_alarm(chat_id, token, args, state).await?,
        &_ => Err(Error::InvalidOption)?,
    }

    Ok(())
}

pub async fn add_tomorrow<'a>(
    chat_id: ChatId,
    token: &str,
    args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let (hour, min, text) = parse_args(chat_id, token, args, state)?;

    state.wakeup.set_tomorrow(hour as u8, min as u8).await;
    send_text(chat_id, token, text)
        .await
        .map_err(|_| Error::CouldNotRespond)?;

    Ok(())
}

pub async fn add_usually<'a>(
    chat_id: ChatId,
    token: &str,
    args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let (hour, min, text) = parse_args(chat_id, token, args, state)?;

    state.wakeup.set_usually(hour as u8, min as u8).await;
    send_text(chat_id, token, text)
        .await
        .map_err(|_| Error::CouldNotRespond)?;

    Ok(())
}

fn parse_args<'a>(
    chat_id: ChatId,
    token: &str,
    mut args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(u8, u8, String), Error> {
    let time = args.next().ok_or(Error::NoTimeArgument)?;

    //assumes hh:mm
    let mut time = time.split(':');
    let hour = time
        .next()
        .ok_or(Error::WrongTimeFormat)?
        .parse::<u32>()
        .map_err(|_| Error::InvalidHourArgument)?;
    let min = time
        .next()
        .ok_or(Error::WrongTimeFormat)?
        .parse::<u32>()
        .map_err(|_| Error::InvalidHourArgument)?;

    if hour > 24 {
        return Err(Error::InvalidHourArgument);
    }
    if min > 60 {
        return Err(Error::InvalidMinuteArgument);
    }

    let now = Local::now();
    let mut alarm = now
        .with_hour(hour).unwrap()
        .with_minute(min).unwrap();

    if alarm < now {
        let tomorrow = now.date().succ();
        alarm = tomorrow.and_hms(hour, min, 0);
    }

    let till_alarm = alarm-now;

    let text = format!(
        "set alarm for over {} hours and {} minutes from now",
        till_alarm.num_hours(),
        till_alarm.num_minutes() % 60
    );

    Ok((hour as u8, min as u8, text))
}

pub async fn list_alarm(chat_id: ChatId, token: &str, state: &State) -> Result<(), Error> {
    let alarms = state.jobs.list();
    let mut list = String::with_capacity(alarms.len() * 100);

    for (id, alarm) in alarms.into_iter() {
        list.push_str(&format!(
            "{:x}, {}, {:?}, {:?}",
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

    send_text(chat_id, token, list)
        .await //.unwrap();
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}

pub async fn remove_alarm<'a>(
    chat_id: ChatId,
    token: &str,
    mut args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let to_remove = dbg!(args.next()).ok_or(Error::NoAlarmId)?;
    let to_remove = u64::from_str_radix(to_remove, 16).unwrap();

    let removed = state.jobs.remove_alarm(to_remove).unwrap(); //TODO //FIXME
                                                               //.map_err(|_| Error::CouldNotRemove)?;

    let text = if let Some(alarm) = removed {
        format!(
            "removed alarm {:x}: {}, {:?}, {:?}",
            to_remove,
            &alarm.time.with_timezone(&Local).to_rfc2822(),
            &alarm.action,
            &alarm.expiration,
        )
    } else {
        String::from("Alarm does not exist, no alarm removed")
    };

    send_text(chat_id, token, text)
        .await //.unwrap();
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}
