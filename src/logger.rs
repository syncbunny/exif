const LOGLEVEL_DEBUG:i32 = 0;
const LOGLEVEL_INFO:i32 = 1;
const LOGLEVEL_WARN:i32 = 2;
const LOGLEVEL_CRIT:i32 = 3;

static loglevel:i32 = LOGLEVEL_INFO;

pub fn debug(msg:&str) {
    if loglevel <= LOGLEVEL_DEBUG {
        println!("{}", msg);
    }
}

pub fn info(msg:&str) {
    if loglevel <= LOGLEVEL_INFO {
        println!("{}", msg);
    }
}

pub fn warn(msg:&str) {
    if loglevel <= LOGLEVEL_WARN {
        println!("{}", msg);
    }
}

pub fn crit(msg:&str) {
    if loglevel <= LOGLEVEL_CRIT {
        println!("{}", msg);
    }
}