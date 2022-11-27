use super::send_text;
use crate::input::web_api::server::State;
use chrono::{Local, Timelike};
use telegram_bot::types::refs::ChatId;

#[derive(Debug)]
pub enum Error {
    InvalidOption,
    CouldNotRespond,
    NoTimeArgument,
    InvalidHourArgument,
    InvalidMinuteArgument,
    WrongTimeFormat,
    BackendError,
}

pub async fn handle<'a>(
    chat_id: ChatId,
    token: &str,
    mut args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let command = args.next();

    match command {
        None => print_current(chat_id, token, state).await?,
        Some("tomorrow") => add_tomorrow(chat_id, token, args, state).await?,
        Some("usually") => add_usually(chat_id, token, args, state).await?,
        Some("help") => help(chat_id, token).await?,
        _ => Err(Error::InvalidOption)?,
    }

    Ok(())
}

async fn help(
    chat_id: ChatId,
    token: &str,
) -> Result<(), Error> {

    let text = String::from("Check, Set and remove alarms
    * use 'tomorrow' followed by a time to set the alarm only for tomorrow
    * use 'usually' followed by a time to set the alarm for all other days
    * format a time as hh:mm or - to unset
    * without any arguments this displays the tomorrow and usually times");

    send_text(chat_id, token, text)
        .await
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}

async fn add_tomorrow<'a>(
    chat_id: ChatId,
    token: &str,
    args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let Parsed { time, status } = parse_args(chat_id, token, args, state)?;

    state.wakeup.set_tomorrow(time).await.map_err(|_| Error::BackendError)?;
    send_text(chat_id, token, status)
        .await
        .map_err(|_| Error::CouldNotRespond)?;

    Ok(())
}

async fn add_usually<'a>(
    chat_id: ChatId,
    token: &str,
    args: std::str::SplitWhitespace<'a>,
    state: &State,
) -> Result<(), Error> {
    let Parsed { time, status } = parse_args(chat_id, token, args, state)?;

    state.wakeup.set_usually(time).await.map_err(|_| Error::BackendError)?;
    send_text(chat_id, token, status)
        .await
        .map_err(|_| Error::CouldNotRespond)?;

    Ok(())
}

struct Parsed {
    time: Option<(u8,u8)>,
    status: String,
}

fn parse_args<'a>(
    _: ChatId,
    _: &str,
    mut args: std::str::SplitWhitespace<'a>,
    _: &State,
) -> Result<Parsed, Error> {
    use chrono::TimeZone;
    let time = args.next().ok_or(Error::NoTimeArgument)?;

    if time == "-" {
        return Ok(Parsed {
            time: None,
            status: String::from("Alarm unset"),
        })
    }

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
        let alarm = now.date_naive().succ_opt().unwrap()
        .and_hms_opt(hour, min, 0).unwrap().and_local_timezone(Local).unwrap();
    }

    let till_alarm = alarm-now;

    let status = format!(
        "set alarm for over {} hours and {} minutes from now",
        till_alarm.num_hours(),
        till_alarm.num_minutes() % 60
    );

    Ok(Parsed { 
        time: Some((hour as u8, min as u8)), 
        status})
}

pub async fn print_current(chat_id: ChatId, token: &str, state: &State) -> Result<(), Error> {
    let tomorrow = state.wakeup.tomorrow();
    let usually = state.wakeup.usually();

    let msg = match (tomorrow, usually) {
        (Some(tomorrow), Some(usually)) => 
            format!("alarm tomorrow: {:02}:{:02}\nusually: {:02}:{:02}", 
                tomorrow.0, tomorrow.1, 
                usually.0, usually.1),
        (Some(tomorrow), None) => 
            format!("alarm tomorrow: {:02}:{:02}, usually no alarm", 
                tomorrow.0, tomorrow.1),
        (None, Some(usually)) => 
            format!("alarm tomorrow as usual: {:02}:{:02}", 
                usually.0, usually.1),
        (None, None) => String::from("No alarm set"),
    };

    send_text(chat_id, token, msg)
        .await
        .map_err(|_| Error::CouldNotRespond)?;
    Ok(())
}
