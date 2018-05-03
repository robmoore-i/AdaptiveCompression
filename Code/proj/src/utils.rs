#![macro_use]

#[macro_export]
macro_rules! dbg {
        ($s:ident, $fmt:expr, $($arg:tt)*) => {
            if $s {
                (print!(concat!($fmt, "\n"), $($arg)*));
            }
        }
    }

#[macro_export]
macro_rules! t_block {
    ($work:block, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
}

#[macro_export]
macro_rules! t_expr {
    ($work:expr, $tvar:ident) => {
            let start = PreciseTime::now();
            $work;
            let end = PreciseTime::now();
            $tvar = $tvar + start.to(end);
        };
}

// Macro for making hashmaps
// I credit this macro (map) to this bod:
// https://stackoverflow.com/a/27582993/3803302
#[macro_export]
macro_rules! map (
        { $($key:expr => $value:expr),+ } => {
            {
                let mut m = HashMap::new();
                $(
                    m.insert($key, $value);
                )+
                m
            }
        };
    );

// Prints a float vec where each float is to 4dp.
pub fn pretty_println_f64vec(floats: &Vec<f64>) {
    print!("[");
    print!("{}", floats[0]);
    for i in 1..floats.len() {
        print!(", {:.4}", floats[i]);
    }
    print!("]\n");
}