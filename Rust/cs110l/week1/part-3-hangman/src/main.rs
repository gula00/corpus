// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    let mut guessed_letters: Vec<char> = Vec::new();
    let mut incorrect_guesses: u32 = 0;

    println!("Welcome to CS110L Hangman!");

    loop {
        // Print current word state
        let word_so_far: String = secret_word_chars
            .iter()
            .map(|c| if guessed_letters.contains(c) { *c } else { '-' })
            .collect();
        println!("\nThe word so far is {}", word_so_far);

        // Print guessed letters
        let guessed_str: String = guessed_letters.iter().collect();
        println!("You have guessed the following letters: {}", guessed_str);
        println!(
            "You have {} guesses left",
            NUM_INCORRECT_GUESSES - incorrect_guesses
        );

        // Check win condition
        if word_so_far == secret_word {
            println!(
                "\nCongratulations you guessed the secret word: {}!",
                secret_word
            );
            break;
        }

        // Check lose condition
        if incorrect_guesses >= NUM_INCORRECT_GUESSES {
            println!("\nSorry, you ran out of guesses!");
            println!("The secret word was: {}", secret_word);
            break;
        }

        // Get user input
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");
        let guess: char = guess.trim().chars().next().unwrap_or(' ');

        if guessed_letters.contains(&guess) {
            println!("You already guessed that letter!");
            continue;
        }

        guessed_letters.push(guess);

        if secret_word_chars.contains(&guess) {
            println!("Correct!");
        } else {
            println!("Sorry, that letter is not in the word");
            incorrect_guesses += 1;
        }
    }
}
