extern crate search_engine;
extern crate serde;
extern crate serde_json;
extern crate stemmer;

use search_engine::index::disk_inverted_index::DiskInvertedIndex;
use search_engine::parser::document_parser;
use search_engine::paths::search_engine_paths;
use search_engine::processor::query_processor;
use search_engine::reader::read_file;
use search_engine::reader::user_input;
use search_engine::index::positional_inverted_index::PositionalInvertedIndex;
use search_engine::index::k_gram_index::KGramIndex;
use std::collections::HashMap;
use std::fs::File;
use std::fs::read_dir;
use std::fs::remove_file;
use std::io::prelude::*;
use std::path::*;

fn main() {
    let mut index_path = search_engine_paths::initialize_path();

    let mut scheme = "default";
    let mut menu: i32;
    let mut input: String;
    let mut change: bool;
    let mut query_index : bool = false;
    let mut ranked_retrieval : bool = false;

    let mut index = PositionalInvertedIndex::new();
    let mut k_gram_index = KGramIndex::new();

    // Loops lets user select first directory to access
    loop {
        println!("Current Working Directory: {}\n", index_path.display());
        print!("Select directory to access: ");
        input = user_input::read_input();
        if input == ":q" {
            return; // Prematurely ends program
        }
        if input.is_empty() {
            println!("Please input valid directory");
            continue;
        }
        change = search_engine_paths::change_directory(&mut index_path, input.as_str());
        
        // Exit loop if successful change to new directory
        if change {
            break;
        }
    }

    loop {
        println!("Current Working Directory: {}\n", index_path.display());

        println!("Select Mode: ");
        println!("1. Build Index");
        println!("2. Query Index\n");
        println!("3. Quit");

        menu = user_input::read_number_range(1,3);

        if menu == 1 {
            query_index = false;
        } else if menu == 2 {
            query_index = true;
        } else if menu == 3 {
            return; 
        }
        break;
    }

    
    if !query_index {
        println!("Use KGrams? (y/n)");
        let yes_no = user_input::read_yes_no();
        if yes_no {
            k_gram_index.enable_k_gram();
        } else {
            k_gram_index.disable_k_gram();
        }
        
        let directory = read_dir(index_path.clone());
        if let Ok(dir) = directory {
            for entry in dir {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let extension = "bin";
                    if path.extension().unwrap() == extension {
                        remove_file(path).unwrap();
                    }
                }
            }
        }

        println!("Building Index...");
        // Builds new index if directory was changed
        
        // Links document ID's to file names
        let id_file = build_index(&index_path, &mut index, &mut k_gram_index);

        let serialized_id_file = serde_json::to_string(&id_file).unwrap();
        let serialized_kgramindex = serde_json::to_string(&k_gram_index).unwrap();

        
        let id_file_filename = format!("{}/{}", index_path.display(), "id_file.bin");
        let kgram_filename = format!("{}/{}", index_path.display(), "kgram.bin");

        //  TODO: BUILD INDEX FILE HERE
        
        let mut id_file_file = match File::create(&id_file_filename) {
            Err(why) => panic!("Couldn't create {} - {}", &id_file_filename, why),
            Ok(file) => file,
        };

        let mut kgram_file = match File::create(&kgram_filename) {
            Err(why) => panic!("Couldn't create {} - {}", &kgram_filename, why),
            Ok(file) => file,
        };

        id_file_file.write(&serialized_id_file.as_bytes()).expect("Failed to write id file");
        kgram_file.write(&serialized_kgramindex.as_bytes()).expect("Failed to write kgram file");

    }
    
    if query_index {
        let disk_inverted_index_path = index_path.clone();
        let disk_inverted_index = DiskInvertedIndex::new(&disk_inverted_index_path.to_str().unwrap());

        let id_file_filename = format!("{}/{}", index_path.display(), "id_file.bin");
        let kgram_filename = format!("{}/{}", index_path.display(), "kgram.bin");

        let mut id_file_file = File::open(id_file_filename).unwrap();

        let mut id_file_contents = String::new();
        id_file_file.read_to_string(&mut id_file_contents).expect("Failed to read id file");

        let id_file : HashMap<u32, String> = serde_json::from_str(&id_file_contents).unwrap();

        let mut kgram_file = File::open(kgram_filename).unwrap();

        let mut kgram_file_contents = String::new();
        kgram_file.read_to_string(&mut kgram_file_contents).expect("Failed to read kgram file");

        k_gram_index = serde_json::from_str(&kgram_file_contents).expect("Error reading kgram file");
        
        loop {
            println!("Current Working Directory: {}\n", index_path.display());

            println!("Choose Retrieval Method: "); 
            println!("1. Boolean Retrieval");
            println!("2. Ranked Retrieval \n");
            println!("3. Quit");

            menu = user_input::read_number_range(1,3);

            if menu == 1 {
                ranked_retrieval = false;
            } else if menu == 2 {
                ranked_retrieval = true;
            } else if menu == 3 {
                return; 
            }
            break;
        }

        loop {
            println!("Current Working Directory: {}", index_path.display());
            print!("Retrieval Method: ");
            if ranked_retrieval {
                println!("Ranked"); 
                println!("Weighting Scheme: {}",scheme);
            } else {
                println!("Boolean"); 
            }
            print!("Input a Query: ");
            input = user_input::read_input_line();
            println!();

            if !input.starts_with(":") {
                process_query(ranked_retrieval, scheme, &input, &disk_inverted_index, &k_gram_index, &id_file);
            } else {
                if input == ":q" || input == ":quit" {
                   return (); 
                } else if input.starts_with(":o ") || input.starts_with(":open ") {
                    open_file(&index_path, input.as_str());
                } else if input.starts_with(":s ") || input.starts_with(":stem ") {
                    stem_term(input.as_str());
                } else if input.starts_with(":i ") || input.starts_with(":index ") {
                    index_directory(&mut index_path, input.clone());
                } else if input == ":mode r" || input == ":mode ranked" {
                    ranked_retrieval = true; 
                } else if input == ":mode b" || input == ":mode boolean" {
                    ranked_retrieval = false; 
                } else if input == ":scheme d" || input == ":scheme default" {
                    scheme = "default"; 
                } else if input == ":scheme t" || input == ":scheme tfidf" {
                    scheme = "tfidf"; 
                } else if input == ":scheme o" || input == ":scheme okapi" {
                    scheme = "okapi"; 
                } else if input == ":scheme w" || input == ":scheme wacky" {
                    scheme = "wacky"; 
                } else if input == ":v" || input == ":vocab" {
                    print_vocab(&disk_inverted_index);
                } else if input == ":k" || input == ":kgram" {
                    print_kgram(&k_gram_index);
                } else if input == ":h" || input == ":help" {
                    print_help(); 
                } else {
                    println!("Invalid command - Use ':help' to view commands");
                }
            }
        }
    }
}

/*
 * Builds the Positional Inverted Index given the directory containing the files it will read
 * 
 * # Arguments
 * 
 * *`index_path` - The Path Buffer of the full file path the program is observing
 * *`index` - The Postitional Inverted Index that will be built given the directory being observed
 * *`k_gram_index` - The K Gram Index that will be built give the directory being observed
 *
 * # Returns
 * 
 * HashMap that contains the association between each Document ID and the file name
 * 
 */
fn build_index(
    index_path: &PathBuf,
    index: &mut PositionalInvertedIndex,
    k_gram_index: &mut KGramIndex,) -> HashMap<u32, String> {

    let directory = index_path.to_str().expect("Not a valid directory");
    document_parser::build_index(directory.to_string(), index, k_gram_index)
}

/*
 * Processes an inputted query before printing the results of that query
 * 
 * # Arguments
 * 
 * *`input` - The inputted query that will be processed
 * *`index` - The Positional Inverted Index that will be used to process the term
 * *`id_file` - HashMap that contains the association between a Document ID and the file name
 */
fn process_query(
    ranked_retrieval: bool,
    scheme: &str,
    input: &str,
    index: &DiskInvertedIndex,
    k_gram_index: &KGramIndex,
    id_file: &HashMap<u32, String>) {

    println!();
    let results = query_processor::process_query(ranked_retrieval, scheme, input, index, k_gram_index, id_file);
    println!();
    for result in results.clone() {
        println!("Result: {}", result);
    }
    println!();
    if !ranked_retrieval {
        if results.len() != 1 {
            println!("{} Documents", results.len());
        } else {
            println!("{} Document", results.len());
        }
    }
    println!();
}

/*
 * Prints out a term after normalizing and stemming
 *
 * # Arguments
 *
 * *`input` - The term that will be normalized and stemmed
 *
 */
fn stem_term(
    input: &str) {

    let mut stem = input.split_whitespace();
    if stem.size_hint().0 > 2 {
        println!("Invalid token");
    } else {
        let string = stem.nth(1).expect("Not a valid token");
        let results = document_parser::normalize_token(string.to_string());
        let result = results.get(0).expect("Not a valid token");
        println!("{}", result);
    }
}

/*
 * Changes the directory of the index_path to a new specified directory
 *
 * # Arguments
 * 
 * *`index_path` - The PathBuf that is set to the current working directory and will be changed to
 * the new directory
 * *`input` - The new directory that will be changed to
 */
fn index_directory(
    mut index_path: &mut PathBuf,
    input: String) -> bool {
    let input_clone = input.clone();
    let mut string = input_clone.split_whitespace();
    let directory = string.nth(1).expect("Not a valid token");
    search_engine_paths::change_directory(&mut index_path, directory)
}

/*
 * Opens a file and prints out its contents
 *
 * # Arguments
 *
 * *`index_path` - The PathBuffer set to the current working directory containing the file to be
 * read
 * *`input` - User input representing the file that will be read
 */
fn open_file(
    index_path: &PathBuf,
    input: &str) {

    let mut string = input.split_whitespace();
    let file = string.nth(1).expect("Not a valid file");
    let mut file_path = index_path.clone();
    println!("Opening {}", file);
    file_path.push(file);
    if file_path.exists() {
        let document = read_file::read_file(file_path.to_str().expect("Not a valid string"));
        println!("\n{}", document.get_title());
        println!("\n{}", document.get_body());
        println!("\n{}", document.get_url());
        println!();
    } else {
        println!("{} does not exist", file_path.display());
    }
}

/*
 * Prints out all vocabulary terms in the index
 *
 * # Arguments
 * 
 * *`index` - The Positional Inverted Index containing the terms
 */
fn print_vocab(
    index: &DiskInvertedIndex) {
    
    println!("Vocabulary");

}

fn print_kgram(
    kgram: &KGramIndex) {
    
    println!("K Grams");

    let kgrams= kgram.get_k_grams();

    for gram in kgrams.iter() {
        println!("{}", gram);
    }
    println!("Total kgrams: {}", kgrams.len());
}

/*
 * Prints the list of commands
 */
fn print_help(){
    println!("\nHelp: \n");
    println!(":h || :help - Prints this dialog :) ");
    println!(":o FILE || :open FILE - Opens a file to read in the current directory");
    println!(":q || :quit - Quits the Program");
    println!(":index DIRECTORY - Changes directory to specified directory and build index under that directory");
    println!(":stem TERM - Normalizes and applies the stemmer on a specified term");
    println!(":enable kgram || :enable k - Enables K Gram Index when indexing");
    println!(":disable kgram || :disable k - Disables K Gram Index when indexing");
    println!(":mode b || :mode boolean - Use Boolean Retrieval Method");
    println!(":mode r || :mode ranked - Use Ranked Retrieval Method");
    println!(":scheme d || :scheme default - Use Default Weight Scheme in Ranked Retrieval");
    println!(":scheme t || :scheme tdidf - Use 'td-idf' Weight Scheme in Ranked Retrieval");
    println!(":scheme o || :scheme okapi - Use Okapi BM25 Weight Scheme in Ranked Retrieval");
    println!(":scheme w || :scheme wacky - Use Wacky Weight Scheme in Ranked Retrieval");
    println!();
}
