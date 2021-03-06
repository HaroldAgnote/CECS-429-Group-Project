use std::time::{Duration, Instant};
use std::collections::HashSet;
use std::collections::BinaryHeap;
use index::disk_inverted_index::DiskInvertedIndex;
use index::disk_inverted_index::IndexReader;
use classifier::classifier::Classifier;
use classifier::classifier::DocumentClass;
use classifier::classifier::TermClassScore;

pub struct BayesianClassifier<'a> {
    index_disputed: &'a DiskInvertedIndex<'a>,
    index_hamilton: &'a DiskInvertedIndex<'a>,
    index_jay: &'a DiskInvertedIndex<'a>,
    index_madison: &'a DiskInvertedIndex<'a>,
}

impl<'a> BayesianClassifier<'a> {
    pub fn new(index_disputed: &'a DiskInvertedIndex, index_hamilton: &'a DiskInvertedIndex, index_jay: &'a DiskInvertedIndex, index_madison: &'a DiskInvertedIndex) -> BayesianClassifier<'a> {
        BayesianClassifier {
            index_disputed: index_disputed,
            index_hamilton: index_hamilton,
            index_jay: index_jay,
            index_madison: index_madison,
        }
    }

    pub fn build_discriminating_vocab_set(&self, k: u32) -> Vec<TermClassScore> {
        let mut priority_queue: BinaryHeap<TermClassScore> = BinaryHeap::new();

        let time = Instant::now();

        let hamilton_vocabulary = self.index_hamilton.get_vocab();
        for term in &hamilton_vocabulary {
            // println!("Hamilton: {}", term);
            match self.calculate_mutual_information_score(term.clone(), DocumentClass::Hamilton) {
                Ok(score) => {
                    match TermClassScore::new(score, term.clone(), DocumentClass::Hamilton) {
                        Some(score) => {
                            priority_queue.push(score);
                        },
                        None => {
                            // Do nothing.
                        },
                    };
                },
                Err(error) => panic!("There was an error calculating the score for term {}. The error is: {}", term, error),
            };
        }
        let jay_vocabulary = self.index_jay.get_vocab();
        for term in &jay_vocabulary {
            match self.calculate_mutual_information_score(term.clone(), DocumentClass::Jay) {
                Ok(score) => {
                    // println!("Jay: {}", term);
                    match TermClassScore::new(score, term.clone(), DocumentClass::Jay) {
                        Some(score) => {
                            priority_queue.push(score);
                        },
                        None => {
                            // Do nothing.
                        },
                    };
                },
                Err(error) => panic!("There was an error calculating the score for term {}. The error is: {}", term, error),
            };
        }
        let madison_vocabulary = self.index_madison.get_vocab();
        for term in &madison_vocabulary {
            // println!("Madison: {}", term);
            match self.calculate_mutual_information_score(term.clone(), DocumentClass::Madison) {
                Ok(score) => {
                    match TermClassScore::new(score, term.clone(), DocumentClass::Madison) {
                        Some(score) => {
                            priority_queue.push(score);
                        },
                        None => {
                            // Do nothing.
                        },
                    };
                },
                Err(error) => panic!("There was an error calculating the score for term {}. The error is: {}", term, error),
            };
        }
        println!("Time taken to build discriminating vocab: {} seconds. Total number of things in priority_queue: {}", time.elapsed().as_secs(), priority_queue.len());
        let mut discriminating_vocab : Vec<TermClassScore> =  Vec::new();
        let mut counter = 0;
        while let Some(from_priority_queue) = priority_queue.pop() {
            if !discriminating_vocab.contains(&from_priority_queue) {
                discriminating_vocab.push(from_priority_queue);
                counter += 1;
                if counter == k {
                    break;
                }
            }
        }

        discriminating_vocab
    }

    fn get_total_num_documents(&self) -> Result<u32, &'static str> { // Nt (or just N), total number of documents for training set.
        let hamilton_total_num = self.index_hamilton.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 
        let jay_total_num = self.index_jay.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 
        let madison_total_num = self.index_madison.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

        let mut total_num = 0;
        total_num += hamilton_total_num;
        total_num += jay_total_num;
        total_num += madison_total_num;

        match total_num > 0 {
            true => Ok(total_num),
            false => Err("Error: No Documents Found in Index"),
        }
    }

    fn get_n_00(&self, term: &str, class: &DocumentClass) -> Result<u32, &'static str> { // N00, total number of documents that DO NOT contain term t and NOT in class c.
        match *class {
            DocumentClass::Hamilton => {
                let jay_total_num = self.index_jay.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 
                let madison_total_num = self.index_madison.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);
                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let jay_num_without_term = jay_total_num - jay_doc_freq_for_term;
                let madison_num_without_term = madison_total_num - madison_doc_freq_for_term;

                let n_00_hamilton = jay_num_without_term + madison_num_without_term;

                Ok(n_00_hamilton)
            },
            DocumentClass::Jay => {
                let hamilton_total_num = self.index_hamilton.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 
                let madison_total_num = self.index_madison.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);
                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let hamilton_num_without_term = hamilton_total_num - hamilton_doc_freq_for_term;
                let madison_num_without_term = madison_total_num - madison_doc_freq_for_term;

                let n_00_jay = hamilton_num_without_term + madison_num_without_term;

                Ok(n_00_jay)
            },
            DocumentClass::Madison => {
                let hamilton_total_num = self.index_hamilton.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 
                let jay_total_num = self.index_jay.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);
                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);

                let hamilton_num_without_term = hamilton_total_num - hamilton_doc_freq_for_term;
                let jay_num_without_term = jay_total_num - jay_doc_freq_for_term;

                let n_00_madison = hamilton_num_without_term + jay_num_without_term;

                Ok(n_00_madison)
            },
        }
    }

    fn get_n_01(&self, term: &str, class: &DocumentClass) -> Result<u32, &'static str> { // N01, total number of documents that DO contain term t and NOT in class c.
        match *class {
            DocumentClass::Hamilton => {
                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);

                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let n_01_hamilton = jay_doc_freq_for_term + madison_doc_freq_for_term;

                Ok(n_01_hamilton)
            },
            DocumentClass::Jay => {
                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);

                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let n_01_jay = hamilton_doc_freq_for_term + madison_doc_freq_for_term;

                Ok(n_01_jay)
            },
            DocumentClass::Madison => {
                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);

                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);

                let n_01_madison = hamilton_doc_freq_for_term + jay_doc_freq_for_term;

                Ok(n_01_madison)
            },
        }
    }

    fn get_n_10(&self, term: &str, class: &DocumentClass) -> Result<u32, &'static str> { // N10, total number of documents that DO NOT contain term t but IS in class c.
        match *class {
            DocumentClass::Hamilton => {
                let hamilton_total_num = self.index_hamilton.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);

                let hamilton_num_without_term = hamilton_total_num - hamilton_doc_freq_for_term; 

                let n_10_hamilton = hamilton_num_without_term;

                Ok(n_10_hamilton)
            },
            DocumentClass::Jay => {
                let jay_total_num = self.index_jay.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);

                let jay_num_without_term = jay_total_num - jay_doc_freq_for_term;

                let n_10_jay = jay_num_without_term;

                Ok(n_10_jay)
            },
            DocumentClass::Madison => {
                let madison_total_num = self.index_madison.get_num_documents().expect("No Documents found!"); // Nc, total number of documents for class. 

                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let madison_num_without_term = madison_total_num - madison_doc_freq_for_term;

                let n_10_madison = madison_num_without_term;

                Ok(n_10_madison)
            },
        }
    }

    fn get_n_11(&self, term: &str, class: &DocumentClass) -> Result<u32, &'static str> { // N11, total number of documents that DO contain term t and IS in class c.
        match *class {
            DocumentClass::Hamilton => {
                let hamilton_doc_freq_for_term = self.index_hamilton.get_document_frequency(term);

                let n_11_hamilton = hamilton_doc_freq_for_term;

                Ok(n_11_hamilton)
            },
            DocumentClass::Jay => {
                let jay_doc_freq_for_term = self.index_jay.get_document_frequency(term);

                let n_11_jay = jay_doc_freq_for_term;

                Ok(n_11_jay)
            },
            DocumentClass::Madison => {
                let madison_doc_freq_for_term = self.index_madison.get_document_frequency(term);

                let n_11_madison = madison_doc_freq_for_term;

                Ok(n_11_madison)
            },
        }
    }

    fn get_n_0X(&self, term: &str, class: &DocumentClass) -> Result<(u32, u32, u32), &'static str> { // N0X, N00 + N01
        let n_00 = match self.get_n_00(term, class) {
            Ok(n_00) => n_00,
            Err(_) => panic!("Something happened when calculating N00!"),
        };
        let n_01 = match self.get_n_01(term, class) {
            Ok(n_01) => n_01,
            Err(_) => panic!("Something happened when calculating N01!"),
        };

        let n_0X = n_00 + n_01;

        Ok((n_0X, n_00, n_01))
    }

    fn get_n_X0(&self, term: &str, class: &DocumentClass) -> Result<(u32, u32, u32), &'static str> { // NX0, N00 + N10
        let n_00 = match self.get_n_00(term, class) {
            Ok(n_00) => n_00,
            Err(_) => panic!("Something happened when calculating N00!"),
        };
        let n_10 = match self.get_n_10(term, class) {
            Ok(n_10) => n_10,
            Err(_) => panic!("Something happened when calculating N10!"),
        };
        
        let n_X0 = n_00 + n_10;

        Ok((n_X0, n_00, n_10))
    }

    fn get_n_1X(&self, term: &str, class: &DocumentClass) -> Result<(u32, u32, u32), &'static str> { // N1X, N10 + N11
        let n_10 = match self.get_n_10(term, class) {
            Ok(n_10) => n_10,
            Err(_) => panic!("Something happened when calculating N10!"),
        };
        let n_11 = match self.get_n_11(term, class) {
            Ok(n_11) => n_11,
            Err(_) => panic!("Something happened when calculating N11!"),
        };

        let n_1X = n_10 + n_11;

        Ok((n_1X, n_10, n_11))
    }

    fn get_n_X1(&self, term: &str, class: &DocumentClass) -> Result<(u32, u32, u32), &'static str> { // NX1, N01 + N11
        let n_01 = match self.get_n_01(term, class) {
            Ok(n_01) => n_01,
            Err(_) => panic!("Something happened when calculating N01!"),
        };
        let n_11 = match self.get_n_11(term, class) {
            Ok(n_11) => n_11,
            Err(_) => panic!("Something happened when calculating N11!"),
        };

        let n_X1 = n_01 + n_11;

        Ok((n_X1, n_01, n_11))
    }

    fn calculate_mutual_information_score(&self, term: String, class: DocumentClass) -> Result<f64, &'static str> {
        let n_hamilton = self.index_hamilton.get_num_documents().expect("No Documents found!"); 
        let n_jay = self.index_jay.get_num_documents().expect("No Documents found!"); 
        let n_madison = self.index_madison.get_num_documents().expect("No Documents found!"); 
        let n = n_hamilton + n_jay + n_madison;


        let (n_0X, n_00, n_01) = match self.get_n_0X(&term, &class) {
            Ok(value) => value,
            Err(error) => panic!("Something happened when calculating N_0X! Error: {}", error),
        };
        let (n_X0, _, n_10) = match self.get_n_X0(&term, &class) {
            Ok(value) => value,
            Err(error) => panic!("Something happened when calculating N_X0! Error: {}", error),
        };
        let (n_1X, _, n_11) = match self.get_n_1X(&term, &class) {
            Ok(value) => value,
            Err(error) => panic!("Something happened when calculating N_1X! Error: {}", error),
        };
        let (n_X1, _, _) = match self.get_n_X1(&term, &class) {
            Ok(value) => value,
            Err(error) => panic!("Something happened when calculating N_X1! Error: {}", error),
        };

        let first_term_calculation = (n_11 as f64 / n as f64) * ((n * n_11) as f64 / (n_1X * n_X1) as f64).log2();
        let second_term_calculation = (n_10 as f64 / n as f64) * ((n * n_10) as f64 / (n_1X * n_X0) as f64).log2();
        let third_term_calculation = (n_01 as f64 / n as f64) * ((n * n_01) as f64 / (n_0X * n_X1) as f64).log2();
        let fourth_term_calculation = (n_00 as f64 / n as f64) * ((n * n_00) as f64 / (n_0X * n_X0) as f64).log2();

        let first_term = match first_term_calculation.is_nan() {
            true => 0.0,
            false => first_term_calculation,
        };
        let second_term = match second_term_calculation.is_nan() {
            true => 0.0,
            false => second_term_calculation,
        };
        let third_term = match third_term_calculation.is_nan() {
            true => 0.0,
            false => third_term_calculation,
        };
        let fourth_term = match fourth_term_calculation.is_nan() {
            true => 0.0,
            false => fourth_term_calculation,
        };

        let score = first_term + second_term + third_term + fourth_term;

        Ok(score)
    }

    fn calculate_probability_term_given_class(&self, doc_id: u32, index_for_class: &DiskInvertedIndex, discriminating_vocab: &Vec<TermClassScore>) -> f64 {
        let terms_for_doc = self.index_disputed.get_terms_for_document(doc_id);
        let mut discriminating_vocab_hashset = HashSet::new();
        for term_class_score in discriminating_vocab {
            discriminating_vocab_hashset.insert(term_class_score.term.clone());
        }

        let doc_terms_intersected_discriminating_terms: HashSet<_> = terms_for_doc.intersection(&discriminating_vocab_hashset).collect();

        let discriminating_vocab_len = discriminating_vocab.len() as u32;
        
        let mut discriminating_terms_score = 0;
        let mut probability = 0.0;
        
        for discriminating_term in discriminating_vocab {
            discriminating_terms_score += index_for_class.get_term_frequency(&discriminating_term.term);
        }

        for term in doc_terms_intersected_discriminating_terms {
            // println!("Term: {}", term);
            // println!("Term Freq: {}", index_for_class.get_term_frequency(&term) + 1);
            probability += ((index_for_class.get_term_frequency(&term) + 1) as f64/(discriminating_terms_score + discriminating_vocab_len) as f64).log2();
            // println!("Probability from now: {}", probability);
        }

        // println!("Discriminating Terms Score: {}", discriminating_terms_score);
        // println!("Probability: {}", probability);
        probability
    }

}

impl<'a> Classifier<'a> for BayesianClassifier<'a> {
    fn classify(&self, doc_id: u32) -> &'a str {
        let k = 50;

        println!("Using k={}", k);

        let discriminating_vocab = self.build_discriminating_vocab_set(k);

        let discriminating_vocab_len = discriminating_vocab.len() as u32;
        
        let num_docs_hamilton = self.index_hamilton.get_num_documents().unwrap();
        let num_docs_madison = self.index_madison.get_num_documents().unwrap();
        let num_docs_jay = self.index_jay.get_num_documents().unwrap();
        
        let total_num_docs = self.get_total_num_documents().unwrap();

        let p_c_hamilton : f64 = num_docs_hamilton as f64/total_num_docs as f64;
        let p_c_madison : f64 = num_docs_madison as f64/total_num_docs as f64;
        let p_c_jay : f64 = num_docs_jay as f64/total_num_docs as f64;

        let mut hamilton_score : f64 = 0.0;
        let mut madison_score : f64 = 0.0;
        let mut jay_score : f64 = 0.0;

        let mut p_t_c_hamilton : f64 = 0.0;
        let mut p_t_c_madison : f64 = 0.0;
        let mut p_t_c_jay : f64 = 0.0;

        p_t_c_hamilton = self.calculate_probability_term_given_class(doc_id, self.index_hamilton, &discriminating_vocab);
        p_t_c_madison = self.calculate_probability_term_given_class(doc_id, self.index_madison, &discriminating_vocab);
        p_t_c_jay = self.calculate_probability_term_given_class(doc_id, self.index_jay, &discriminating_vocab);

        hamilton_score = (p_c_hamilton).log2() + p_t_c_hamilton;
        madison_score = (p_c_madison).log2() + p_t_c_madison;
        jay_score = (p_c_jay).log2() + p_t_c_jay;

        println!("Hamilton Classification Score: {}", hamilton_score);
        println!("Madison Classification Score: {}", madison_score);
        println!("Jay Classification Score: {}\n", jay_score);

        if hamilton_score > madison_score {
            if hamilton_score > jay_score {
                return "Hamilton" ;
            } else {
                return "Jay" ;
            }
        } else {
            if madison_score > jay_score {
                return "Madison" ;
            }  else {
                return "Jay";
            }
        }
    }
    fn get_all_vocab(&self) -> Vec<String> {
        let vocabulary_hamilton = self.index_hamilton.get_vocab();
        let vocabulary_jay = self.index_jay.get_vocab();
        let vocabulary_madison = self.index_madison.get_vocab();

        let first_union: HashSet<_> = vocabulary_hamilton.union(&vocabulary_jay).collect();
        let mut first_union_final: HashSet<String> = HashSet::new();
        for vocab in first_union {
            first_union_final.insert(vocab.clone());
        }

        let second_union: HashSet<_> = first_union_final.union(&vocabulary_madison).collect();
        let mut final_union: HashSet<String> = HashSet::new();
        for vocab in second_union {
            final_union.insert(vocab.clone());
        }

        let mut vocab_list : Vec<String> = Vec::new();

        for vocab in final_union {
            vocab_list.push(vocab); 
        }
        vocab_list.sort();
        
        vocab_list
    }

}
