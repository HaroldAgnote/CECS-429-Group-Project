use std::cmp::Ordering;
use std::collections::HashMap;

/*
 * Structure that will contain the Positional Posting of a term in the
 * Positional Inverted Index
 */
 #[derive(Clone)]
pub struct PositionalPosting {
    /*
     * Document ID of Positional Posting
     */
    m_doc_id: u32,

    /*
    * Score of term for the given document
    */
    m_term_score: f64,

    m_tf_idf_term_score: f64,

    m_okapi_term_score: f64,

    m_wacky_term_score: f64,

    /*
     * The list of positions for each posting
     */
    m_positions: Vec<u32>,
}

/*
 * Contains operations for Positional Posting
 */
impl PositionalPosting {
    /*
     * Constructor of PositionalPosting
     *
     * # Arguments
     *
     * *`doc_id` - The doc id of the constructed PositionalPosting
     *
     * # Returns
     *
     * The newly constructed PositionalPosting
     *
     */
    pub fn new(doc_id: u32) -> PositionalPosting {
        PositionalPosting {
            m_doc_id: doc_id,
            m_positions: Vec::new(),
            m_term_score: 0.0_f64,
            m_tf_idf_term_score: 0.0f64,
            m_okapi_term_score: 0.0f64,
            m_wacky_term_score: 0.0f64
        }
    }

    /*
     * Returns document ID clone to preserve data integrity
     *
     * # Returns
     *
     * The document id of the posting
     */
    pub fn get_doc_id(&self) -> u32 {
        self.m_doc_id.clone()
    }

    /*
     * Returns weight of term for document clone to preserve data integrity
     *
     * # Returns
     *
     * The weight of the term for the document in the posting
     */
    pub fn get_term_score(&self) -> f64 {
        self.m_term_score.clone()
    }

    pub fn get_tf_idf_term_score(&self) -> f64 {
        self.m_tf_idf_term_score.clone()
    }

    pub fn get_okapi_term_score(&self) -> f64 {
        self.m_okapi_term_score.clone()
    }

    pub fn get_wacky_term_score(&self) -> f64 {
        self.m_wacky_term_score.clone()
    }
    /*
     * Returns positions of term for the posting.
     * 
     * # Returns
     *
     * Clone of posting's positions to preserve data integrity
     */
    pub fn get_positions(&self) -> Vec<u32> {
        self.m_positions.clone()
    }

    /*
     * Adds a new position for the posting
     *
     * # Arguments
     *
     * *`pos` - The position to be added to the Posting
     */
    pub fn add_position(&mut self, pos: u32) {
        self.m_positions.push(pos);
    }

    /*
     * Returns the last position the posting was listed
     *
     * # Returns
     *
     * The last position added to the posting
     */
    fn get_last_position(&self) -> u32 {
        let pos: u32 = self.m_positions
            .last()
            .expect("Not a valid position")
            .clone();
        pos
    }
}

impl Ord for PositionalPosting {
    fn cmp(&self, other: &PositionalPosting) -> Ordering {
        self.m_doc_id.cmp(&other.m_doc_id)    
    }
}

impl Eq for PositionalPosting {
}

impl PartialOrd for PositionalPosting{
    fn partial_cmp(&self, other: &PositionalPosting) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PositionalPosting{
    fn eq(&self, other: &PositionalPosting) -> bool {
        self.m_doc_id == other.m_doc_id
    }
}
/*
 * Representation of a Positional Inverted Index
 */
pub struct PositionalInvertedIndex {
    /*
     * Holds terms processed by index and the positional 
     * postings each term is mapped to
     */
    m_index: HashMap<String, Vec<PositionalPosting>>,
}

/*
 * Contains implemented operations of the Positional Inverted Index
 */
impl PositionalInvertedIndex {
    /*
     * Constructs Positional Inverted Index 
     *
     * # Returns
     *
     * Newly constructed PositionalInvertedIndex
     */
    pub fn new() -> PositionalInvertedIndex {
        PositionalInvertedIndex { m_index: HashMap::new() }
    }

    /*
     * Adds a term to the Positional Inverted Index and 
     * an associated Positional Posting to its
     * list (doc_id and position).
     *
     * # Arguments
     *
     * *`term` - The term to be added to the index
     * *`doc_id` - The document id where this term exists
     * *`pos` - The marked position of the term in the document
     *
     */
    pub fn add_term(&mut self, term: &str, doc_id: u32, pos: u32) {
        if self.m_index.contains_key(term) {
            {
                let positional_postings = self.m_index.get_mut(term)
                    .expect("No term found");

                let num_of_doc_ids = positional_postings.len();
                let last_posting = positional_postings.get_mut(num_of_doc_ids - 1)
                    .expect(
                    "Could not get posting",
                );

                if last_posting.get_doc_id() == doc_id {
                    let last_position = last_posting.get_last_position();
                    if last_position != pos {
                        last_posting.add_position(pos);
                    }
                }
            }
            {
                if self.m_index.get(term).unwrap().last().unwrap().get_doc_id() != doc_id {
                    let mut new_posting = PositionalPosting::new(doc_id);
                    new_posting.add_position(pos);
                    self.m_index.get_mut(term).expect("term not found").push(
                        new_posting,
                    );
                }
            }
        } else {
            let mut new_posting = PositionalPosting {
                m_doc_id: doc_id,
                m_positions: Vec::new(),
                m_term_score: 0.0f64,
                m_tf_idf_term_score: 0.0f64,
                m_okapi_term_score: 0.0f64,
                m_wacky_term_score: 0.0f64
            };
            new_posting.add_position(pos);
            let mut positional_postings = Vec::new();
            positional_postings.push(new_posting);
            self.m_index.insert(term.to_string(), positional_postings);
        }
    }

    /*
     * Checks if the index contains an indexed term
     *
     * # Arguments
     * *`term` - The term that will be checked
     *
     * # Returns
     * True if the term exist in the index
     * False otherwise
     */
    pub fn contains_term(&self, term: &str) -> bool {
        self.m_index.contains_key(term)
    }

    /*
     * Returns the postings associated with a term
     *
     * # Arguments
     *
     * *`term` - The term where the postings shall be retrieved
     *
     * # Returns
     * 
     * Vector containing the Positional Postings of a term in the index
     */
    pub fn get_postings(&self, term: &str) -> &Vec<PositionalPosting> {
        self.m_index.get(term).expect("Improper postings")
    }

    /*
     * Returns the number of terms in the index
     *
     * # Returns
     * The number of terms in the index
     */
    pub fn get_term_count(&self) -> usize {
        self.m_index.len()
    }

    /*
     * Returns a sorted dictionary of the terms existing in the index
     *
     * # Returns
     *
     * Returns the terms of the index in sorted order
     */
    pub fn get_dictionary(&self) -> Vec<&String> {
        let mut dictionary = Vec::new();

        for term in self.m_index.keys() {
            dictionary.push(term)
        }
        dictionary.sort();
        return dictionary;
    }


    pub fn set_score(&mut self, term: &str, weight:f64) {
        let m_index = &mut self.m_index;
        m_index.get_mut(term).unwrap().iter_mut().last().unwrap().m_term_score = weight;
    }

    pub fn set_tf_idf_score(&mut self, term: &str, weight:f64) {
        let m_index = &mut self.m_index;
        m_index.get_mut(term).unwrap().iter_mut().last().unwrap().m_tf_idf_term_score = weight;
    }

    pub fn set_okapi_score(&mut self, term: &str, weight:f64) {
        let m_index = &mut self.m_index;
        m_index.get_mut(term).unwrap().iter_mut().last().unwrap().m_okapi_term_score = weight;
    }

    pub fn set_wacky_score(&mut self, term: &str, weight:f64) {
        let m_index = &mut self.m_index;
        m_index.get_mut(term).unwrap().iter_mut().last().unwrap().m_wacky_term_score = weight;
    }
    
}
