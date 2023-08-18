// use::std::io::{Cursor, BufReader, BufRead};

// macro_rules! push_fmt {
//     ($text:expr, $($arg:tt)*) => {
//         $text.push_str(&format!($($arg)*));
//     };
// }

// const TAB: &str = super::KoopaTextBuilder::TAB;

// fn is_label(line: &str) -> bool {
//     line.ends_with(':')
// }

// pub fn post_process(text: String) -> String {
//     let mut res = String::with_capacity(text.len());
//     let reader = BufReader::new(Cursor::new(text));
//     let mut iter = reader.lines().peekable();

//     while let Some(Ok(line1)) = iter.next() {
//         push_fmt!(res, "{line1}\n");
//         if let Some(Ok(line2)) = iter.peek() {
//             if is_label(&line1) && is_label(&line2) {
//                 push_fmt!(
//                     res, "{TAB}br 0, {}, {}\n",
//                     &line1[..line1.len() - 1],
//                     &line2[..line2.len() - 1],    
//                 );
//             }
//         }
//     }

//     res
// }