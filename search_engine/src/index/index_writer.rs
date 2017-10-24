use index::positional_inverted_index::PositionalInvertedIndex;

struct IndexWriter<'a> {
    folder_path: &'a str
}

trait DiskIndex {
    fn get_folder_path(&self) -> &str;

    fn build_index(&self, index: &PositionalInvertedIndex) {
        self.build_index_for_directory(index, self.get_folder_path());
    }

    fn build_index_for_directory(&self, index: &PositionalInvertedIndex, folder: &str);
    fn build_vocab_file(&self, folder: &str, dictionary: &Vec<&String>, vocab_positions: &Vec<u32>);
    fn build_postings_file(&self, folder: &str, index: &PositionalInvertedIndex, dictionary: &Vec<&String>, vocab_positions: &Vec<u32>);
}

impl<'a> IndexWriter<'a> {
    fn new(&self, folder_path: &'a str) -> IndexWriter {
        IndexWriter { folder_path: folder_path }
    }
}

impl<'a> DiskIndex for IndexWriter<'a> {
    fn get_folder_path(&self) -> &str {
        self.folder_path
    }

    fn build_index_for_directory(&self, index: &PositionalInvertedIndex, folder: &str) {
        let dictionary = index.get_dictionary();
        let vocab_positions = Vec::new();
        self.build_vocab_file(folder, &dictionary, &vocab_positions);
        self.build_postings_file(folder, index, &dictionary, &vocab_positions);
    }
    
    fn build_vocab_file(&self, folder: &str, dictionary: Vec<&String>, vocab_positions: Vec<u32>) {

    }

    fn build_postings_file(&self, folder: &str, index: PositionalInvertedIndex, dictionary: Vec<&String>, vocab_positions: &Vec<u32>) {

    }

}