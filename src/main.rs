//  ************************************************************  Prelude contents  ************************************************************


//      std::marker::{Copy, Send, Sized, Sync, Unpin}, marker traits that indicate fundamental properties of types.
//      std::ops::{Drop, Fn, FnMut, FnOnce}, various operations for both destructors and overloading ().
//      std::mem::drop, a convenience function for explicitly dropping a value.
//      std::boxed::Box, a way to allocate values on the heap.
//      std::borrow::ToOwned, the conversion trait that defines to_owned, the generic method for creating an owned type from a borrowed type.
//      std::clone::Clone, the ubiquitous trait that defines clone, the method for producing a copy of a value.
//      std::cmp::{PartialEq, PartialOrd, Eq, Ord}, the comparison traits, which implement the comparison operators and are often seen in trait bounds.
//      std::convert::{AsRef, AsMut, Into, From}, generic conversions, used by savvy API authors to create overloaded methods.
//      std::default::Default, types that have default values.
//      std::iter::{Iterator, Extend, IntoIterator, DoubleEndedIterator, ExactSizeIterator}, iterators of various kinds.
//      std::option::Option::{self, Some, None}, a type which expresses the presence or absence of a value. This type is so commonly used, its variants are also exported.
//      std::result::Result::{self, Ok, Err}, a type for functions that may succeed or fail. Like Option, its variants are exported as well.
//      std::string::{String, ToString}, heap-allocated strings.
//      std::vec::Vec, a growable, heap-allocated vector.



//  ************************************************************        Parse       ************************************************************


//          pub fn parse<F>(&self) -> Result<F, <F as FromStr>::Err>

//      Parses this string slice into another type.

//      Because parse is so general, it can cause problems with type inference. 
//      As such, parse is one of the few times you’ll see the syntax affectionately known as the ‘turbofish’: ::<>. 
//      This helps the inference algorithm understand specifically which type you’re trying to parse into.

//      parse can parse into any type that implements the FromStr trait.


use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..101);

    println!("The secret number is: {}", secret_number);

    loop {

        println!("Please input your guess.");

        let mut guess = String::new();  //changeable    &   let guess = 5; -> immutable

        io::stdin()  //standard input handler
            .read_line(&mut guess)
            .expect("Failed to read line");  //Resault: OK || Err; if Err -> expect show error message

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }

    }
}
