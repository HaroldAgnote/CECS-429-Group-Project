pub struct QueryParser {}

impl QueryParser {
    pub fn new() -> QueryParser {
        QueryParser {}
    }

    /*
     * Takes in a query as a string. E.g. "hello (one + two)"
     * Returns the expanded query as a vector, where each element in the vector is an AND query,
     * and in between the elements is considered an OR query.
     * Thus, ["element1", "element2"] is considered element1 OR element2
     * E.g. ["hello one", "hello two"]
     */
    pub fn process_query(&self, input: &str) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        // Part 1 - Group the token into groups separated by "+"
        let mut tokens = input.split_whitespace();
        let mut query_builder: Vec<String> = Vec::new();

        while let Some(mut token) = tokens.next() {
            if token.len() == 1 && token.starts_with("+") {
                if query_builder.len() != 0 {
                    results.push(query_builder.join(" "));
                    query_builder.clear();
                }
                continue;
            }
            if token.starts_with("(") {
                // Start processing inner queries...
                let mut parenthesis_query_vec = Vec::new();
                let previous_token = query_builder.join(" ");
                query_builder.clear();
                parenthesis_query_vec.push(previous_token);
                parenthesis_query_vec.push(String::from(token));
                token = tokens.next().unwrap();
                let mut left_parenthesis_counter = 1;
                while left_parenthesis_counter != 0 {
                    parenthesis_query_vec.push(String::from(token));
                    if token.starts_with("(") {
                        left_parenthesis_counter += 1;
                    }
                    if token.ends_with(")") {
                        let mut reverse_token_iter = token.chars().rev();
                        while let Some(c) = reverse_token_iter.next() {
                            if c == ')' {
                                left_parenthesis_counter -= 1;
                            } else {
                                break;
                            }
                        }
                    }
                    match tokens.next() {
                        Some(part) => token = part,
                        None => break,
                    }
                }
                results.push(parenthesis_query_vec.join(" "));
                continue;
            }
            query_builder.push(String::from(token));
        }
        if query_builder.len() != 0 {
            results.push(query_builder.join(" "));
        }

        // Part 2 - Expand inner queries if any...
        let mut final_results: Vec<String> = Vec::new();
        let mut results_iter = results.iter();
        while let Some(result) = results_iter.next() {
            if result.ends_with(")") {
                // I should find a better way of checking if it's nested...
                // Process the result...
                let sub_results = self.multiply_query(result);
                // Add sub-results to final results...
                for sub_result in sub_results {
                    final_results.push(sub_result);
                }
                continue;
            }
            final_results.push(result.clone());
        }
        final_results
    }

    /*
     * Given a string query, multiplies them out, recursively.
     * E.g. hello (one + two) -> ["hello one", "hello two"]
     */
    pub fn multiply_query(&self, input: &str) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        let mut tokens = input.split_whitespace();
        let mut multiplier_vec: Vec<String> = Vec::new();
        let mut multiplicand_vec: Vec<String> = Vec::new();

        let mut query_builder: Vec<String> = Vec::new();

        // Extract the multiplier
        while let Some(token) = tokens.next() {
            if token.starts_with("(") {
                query_builder.push(String::from(token));
                break;
            }
            multiplier_vec.push(String::from(token));
        }

        // Extract the multiplicand
        while let Some(mut token) = tokens.next() {
            if token.len() == 1 && token.starts_with("+") {
                if query_builder.len() != 0 {
                    multiplicand_vec.push(query_builder.join(" "));
                    query_builder.clear();
                }
                continue;
            }
            if token.starts_with("(") {
                query_builder.push(String::from(token));
                token = tokens.next().unwrap();
                let mut left_parenthesis_counter = 1;
                while left_parenthesis_counter != 0 {
                    query_builder.push(String::from(token));
                    if token.starts_with("(") {
                        left_parenthesis_counter += 1;
                    }
                    if token.ends_with(")") {
                        let mut reverse_token_iter = token.chars().rev();
                        while let Some(c) = reverse_token_iter.next() {
                            if c == ')' {
                                left_parenthesis_counter -= 1;
                            } else {
                                break;
                            }
                        }
                    }
                    match tokens.next() {
                        Some(part) => token = part,
                        None => break,
                    }
                }
                multiplicand_vec.push(query_builder.join(" "));
                query_builder.clear();
                continue;
            }
            query_builder.push(String::from(token)); // Finish the job...
        }
        // Final clean up...
        if query_builder.len() != 0 {
            multiplicand_vec.push(query_builder.join(" "));
            query_builder.clear();
        }

        let multiplier = multiplier_vec.join(" ");
        multiplicand_vec[0] = multiplicand_vec[0].chars().skip(1).collect();
        let multiplicand_vec_length = multiplicand_vec.len();
        let multiplicand_last_element_length = multiplicand_vec[multiplicand_vec_length - 1].len();
        multiplicand_vec[multiplicand_vec_length - 1] = multiplicand_vec[multiplicand_vec_length -
                                                                             1]
            .chars()
            .take(multiplicand_last_element_length - 1)
            .collect();
        for multiplicand in multiplicand_vec {
            results.push(multiplier.clone() + " " + multiplicand.as_str());
        }
        let mut final_results: Vec<String> = Vec::new();
        let mut results_iter = results.iter();
        while let Some(result) = results_iter.next() {
            if result.ends_with(")") {
                // I should find a better way of checking if it's nested...
                // Process the result...
                let sub_results = self.multiply_query(result);
                // Add sub-results to final results...
                for sub_result in sub_results {
                    final_results.push(sub_result);
                }
                continue;
            }
            final_results.push(result.clone());
        }
        final_results
    }
}
