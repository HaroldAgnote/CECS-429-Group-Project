use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct DiskInvertedIndex<'a> {
    path: &'a str,
    vocab_list: File,
    pub postings: File,
    vocab_table: Vec<u64>,
}

pub trait IndexReader {
    fn read_postings_from_file(&self, mut postings: &File, postings_position: i64) -> Vec<u32>;
    fn get_positions(&self, term: &str) -> HashMap<u32, Vec<u32>>; // Map from document ids to term positions for a document id
    fn get_postings(&self, term: &str) -> Result<Vec<u32>, &'static str>;
    fn contains_term(&self, term: &str) -> bool;
    fn get_document_score(&self, term: &str, doc_id_wanted: u32) -> Option<f64>;
    fn get_document_frequency(&self, term: &str) -> u32;
    fn get_term_frequency(&self, term: &str, doc_id_wanted: u32) -> Option<u32>;
    fn get_document_length(&self, term: &str) -> f64;
    fn binary_search_vocabulary(&self, term: &str) -> i64;
    fn read_vocab_table(index_name: &str) -> Vec<u64>;
    fn get_term_count(&self) -> u32;
}

impl<'a> DiskInvertedIndex<'a> {
    pub fn new(path: &'a str) -> DiskInvertedIndex {
        DiskInvertedIndex {
            path: path,
            vocab_list: File::open(format!("{}/{}", path, "vocab.bin")).unwrap(),
            postings: File::open(format!("{}/{}", path, "postings.bin")).unwrap(),
            vocab_table: DiskInvertedIndex::read_vocab_table(path),
        }
    }
}

impl<'a> IndexReader for DiskInvertedIndex<'a> {
    fn read_postings_from_file(&self, mut postings: &File, postings_position: i64) -> Vec<u32> {
        let mut doc_ids: Vec<u32> = Vec::new();
        postings.seek(SeekFrom::Start(postings_position as u64)).unwrap();
        let mut doc_freq_buffer = [0; 4]; // Four bytes of 0.
        postings.read_exact(&mut doc_freq_buffer);
        println!("Document Frequency: {}", (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap());
        let document_frequency = (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
        let mut doc_id = 0;
        for i in 0..document_frequency {
            let mut doc_id_buffer = [0; 4];
            postings.read_exact(&mut doc_id_buffer);
            println!("Document Id: {}", (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap());
            doc_id += (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap();
            doc_ids.push(doc_id);

            let mut doc_score_buffer = [0; 8];
            postings.read_exact(&mut doc_score_buffer);
            println!("Document Score: {}", (&doc_score_buffer[..]).read_f64::<BigEndian>().unwrap());
            // Do something with document score here

            let mut term_freq_buffer = [0; 4];
            postings.read_exact(&mut term_freq_buffer);
            println!("Term Frequency: {}", (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap());
            let term_frequency = (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
            postings.seek(SeekFrom::Current((term_frequency * 4) as i64)); // Skip reading term positions... We only need doc ids.
        }
        doc_ids
    }

    fn get_positions(&self, term: &str) -> HashMap<u32, Vec<u32>> {
        let postings_position = self.binary_search_vocabulary(term);
        let mut doc_to_positions = HashMap::new();
        (&self.postings).seek(SeekFrom::Start(postings_position as u64));
        let mut doc_freq_buffer = [0; 4]; // Four bytes of 0.
        (&self.postings).read_exact(&mut doc_freq_buffer);
        let document_frequency = (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
        let mut doc_id = 0;
        for i in 0..document_frequency {
            let mut doc_id_buffer = [0; 4];
            (&self.postings).read_exact(&mut doc_id_buffer);
            doc_id += (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap();
            println!("This is document ID: {} for occurance {}", doc_id, i);

            let mut positions = Vec::new();

            (&self.postings).seek(SeekFrom::Current(8)); // Skip Document Score (Wdt)
            let mut term_freq_buffer = [0; 4];
            (&self.postings).read_exact(&mut term_freq_buffer);
            let term_frequency = (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
            println!("Term frequency {} for document ID {}", term_frequency, doc_id);
            let mut positions_buffer = [0; 4];
            let mut postings_accumulator = 0;
            for j in 0..term_frequency {
                (&self.postings).read_exact(&mut positions_buffer);
                postings_accumulator += (&positions_buffer[..]).read_u32::<BigEndian>().unwrap();
                println!("Current position: {} for term frequency occurance {}", postings_accumulator, j);
                positions.push(postings_accumulator);
            }
            doc_to_positions.insert(doc_id, positions);
        }
        doc_to_positions
    }

    fn get_document_score(&self, term: &str, doc_id_wanted: u32) -> Option<f64> {
        let postings_position = self.binary_search_vocabulary(term);
        (&self.postings).seek(SeekFrom::Start(postings_position as u64));
        let mut doc_freq_buffer = [0; 4];
        (&self.postings).read_exact(&mut doc_freq_buffer);
        let document_frequency = (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
        let mut doc_id = 0;
        for i in 0..document_frequency {
            let mut doc_id_buffer = [0; 4];
            (&self.postings).read_exact(&mut doc_id_buffer);
            doc_id += (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap();
            let mut document_weight_buffer = [0; 8];
            (&self.postings).read_exact(&mut document_weight_buffer);
            let document_weight = (&document_weight_buffer[..]).read_f64::<BigEndian>().unwrap();
            if doc_id == doc_id_wanted {
                return Some(document_weight);
            }
            let mut term_freq_buffer = [0; 4];
            (&self.postings).read_exact(&mut term_freq_buffer);
            let term_frequency = (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
            (&self.postings).seek(SeekFrom::Current((term_frequency * 4) as i64));
        }
        None
    }

    fn get_document_frequency(&self, term: &str) -> u32 {
        let postings_position = self.binary_search_vocabulary(term);
        (&self.postings).seek(SeekFrom::Start(postings_position as u64));
        let mut doc_freq_buffer = [0; 4];
        (&self.postings).read_exact(&mut doc_freq_buffer);
        (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap() // Return the document frequency
    }

    fn get_term_frequency(&self, term: &str, doc_id_wanted: u32) -> Option<u32> {
        let postings_position = self.binary_search_vocabulary(term);
        (&self.postings).seek(SeekFrom::Start(postings_position as u64));
        let mut doc_freq_buffer = [0; 4];
        (&self.postings).read_exact(&mut doc_freq_buffer);
        let document_frequency = (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
        let mut doc_id = 0;
        for i in 0..document_frequency {
            let mut doc_id_buffer = [0; 4];
            (&self.postings).read_exact(&mut doc_id_buffer);
            doc_id += (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap();
            (&self.postings).seek(SeekFrom::Current(8)); // Skip document weight
            let mut term_freq_buffer = [0; 4];
            (&self.postings).read_exact(&mut term_freq_buffer);
            let term_frequency = (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
            if doc_id == doc_id_wanted {
                return Some(term_frequency);
            }
            (&self.postings).seek(SeekFrom::Current((term_frequency * 4) as i64));
        }
        None
    }

    fn get_document_length(&self, term: &str) -> f64 {
        let postings_position = self.binary_search_vocabulary(term);
        (&self.postings).seek(SeekFrom::Start(postings_position as u64));
        let mut doc_freq_buffer = [0; 4];
        (&self.postings).read_exact(&mut doc_freq_buffer);
        let document_frequency = (&doc_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
        let mut doc_id = 0;
        let mut document_length = 0_f64;
        for i in 0..document_frequency {
            let mut doc_id_buffer = [0; 4];
            (&self.postings).read_exact(&mut doc_id_buffer);
            doc_id += (&doc_id_buffer[..]).read_u32::<BigEndian>().unwrap();
            let mut document_weight_buffer = [0; 8];
            (&self.postings).read_exact(&mut document_weight_buffer);
            let document_weight = (&document_weight_buffer[..]).read_f64::<BigEndian>().unwrap();
            document_length += document_weight.powi(2);
            let mut term_freq_buffer = [0; 4];
            (&self.postings).read_exact(&mut term_freq_buffer);
            let term_frequency = (&term_freq_buffer[..]).read_u32::<BigEndian>().unwrap();
            (&self.postings).seek(SeekFrom::Current((term_frequency * 4) as i64));
        }
        document_length
    }

    fn get_postings(&self, term: &str) -> Result<Vec<u32>, &'static str> {
        println!("get postings function called");
        let postings_position = self.binary_search_vocabulary(term);
        match postings_position >= 0 {
            true => Ok(self.read_postings_from_file(&self.postings, postings_position)),
            false => Err("Postings position is less than 0."),
        }
    }

    fn contains_term(&self, term: &str) -> bool {
        return self.binary_search_vocabulary(term) != -1; 
    }
    fn binary_search_vocabulary(&self, term: &str) -> i64 {
        let mut vocab_list = &self.vocab_list;
        let mut i = 0;
        let mut j = self.vocab_table.len() / 2 - 1;
        while i <= j {
            let m = (i + j) / 2;
            println!("i: {}, j: {}, m: {}", i, j, m);
            let vocab_list_position = self.vocab_table.get(m * 2).unwrap();
            let mut term_length = 0;
            if m == self.vocab_table.len() / 2 - 1 {
                term_length = vocab_list.metadata().unwrap().len() as u64 - self.vocab_table[m * 2];
            }
            else {
                term_length = self.vocab_table.get((m + 1) * 2).unwrap() - vocab_list_position;
            }

            vocab_list.seek(SeekFrom::Start(*vocab_list_position as u64)).unwrap();

            let mut buffer = vec![0; term_length as usize];
            vocab_list.read_exact(&mut buffer);

            let file_term = String::from_utf8(buffer).unwrap();

            let compare_value = term.cmp(&file_term);

            match compare_value {
                Ordering::Equal => return *(self.vocab_table.get(m * 2 + 1)).unwrap() as i64,
                Ordering::Less => j = m - 1,
                Ordering::Greater => i = m + 1
            }
        }
        -1
    }

    fn read_vocab_table(index_name: &str) -> Vec<u64> {
        let mut table_file = File::open(format!("{}/{}", index_name, "vocab_table.bin")).unwrap();
        let mut vocab_size_buffer= [0; 4];
        table_file.read_exact(&mut vocab_size_buffer);
        
        let mut table_index = 0;
        let mut vocab_table = vec![0; (&vocab_size_buffer[..]).read_u32::<BigEndian>().unwrap() as usize * 2];
        let mut vocab_pos_buffer = [0; 8];
        loop {
            match table_file.read_exact(&mut vocab_pos_buffer) {
                Ok(_) => {
                    vocab_table.push((&vocab_pos_buffer[..]).read_u64::<BigEndian>().unwrap());
                    //table_file.seek(SeekFrom::Current(8)); // Skip the document weights (Ld) values...
                    table_index += 1;
                },
                Err(_) => break
            }
        }
        vocab_table
    }

    fn get_term_count(&self) -> u32 {
        return self.vocab_table.len() as u32 / 2;
    }
}
