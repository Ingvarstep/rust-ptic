use regex::Regex;

fn whitespace_tokenize(text: &str) -> Vec<&str> {
    let seperator = Regex::new(r"([ ,.]+)").expect("Invalid regex");
    let tokens = seperator.split(text);
    let vec = tokens.collect::<Vec<&str>>();
    vec
}

fn read_file(file_name: &str) -> std::string::String {
    let contents =
        std::fs::read_to_string(file_name).expect("Something went wrong reading the file");
    contents
}

fn split_lines(text: &std::string::String) -> Vec<&str> {
    let lines = text.lines();
    let vec = lines.collect::<Vec<&str>>();
    vec
}

fn tokenize_lines(lines: Vec<&str>) -> Vec<Vec<&str>> {
    let mut vec = Vec::new();
    for line in lines {
        let tokens = whitespace_tokenize(&line);
        vec.push(tokens);
    }
    vec
}

fn get_word_stat(tokenized_texts: &Vec<Vec<&str>>) -> std::collections::HashMap<String, f64> {
    let mut word2text_count = std::collections::HashMap::new();
    for text in tokenized_texts {
        let uniquewords = text.iter().collect::<std::collections::HashSet<_>>();
        for word in uniquewords {
            let count: &mut f64 = word2text_count.entry(word.to_lowercase()).or_insert(0.);
            *count += 1.;
        }
    }
    word2text_count
}

fn get_class_stat(targets: &Vec<&str>) -> std::collections::HashMap<String, f64> {
    let mut target2count = std::collections::HashMap::new();
    for target in targets {
        let count: &mut f64 = target2count
            .entry(target.to_lowercase().to_string())
            .or_insert(0.);
        *count += 1.;
    }
    target2count
}

fn add_class(
    count_dict: &mut std::collections::HashMap<String, std::collections::HashMap<String, f64>>,
    ts: &std::collections::HashSet<&&str>,
) {

    for t in ts {
        let mut new_map = std::collections::HashMap::new();
        count_dict.insert(t.to_string(), new_map);
    }
}

fn add_word(
    word2count: &mut std::collections::HashMap<String, std::collections::HashMap<String, f64>>,
    t: &str,
    word: &str,
) {
    let count_dict_t = word2count.get_mut(t).unwrap();
    let count: &mut f64 = count_dict_t.entry(word.to_lowercase()).or_insert(0.);
    *count += 1.;
}

fn create_pmi_dict(
    tokenized_texts: &Vec<Vec<&str>>,
    targets: &Vec<&str>,
    min_count: i32,
) -> std::collections::HashMap<String, std::collections::HashMap<String, f64>> {
    let mut count_dict: std::collections::HashMap<String, std::collections::HashMap<String, f64>> =
        std::collections::HashMap::new();
    let mut pmi_dict: std::collections::HashMap<String, std::collections::HashMap<String, f64>> =
        std::collections::HashMap::new();
    let ts = targets.iter().collect::<std::collections::HashSet<_>>();
    let target2count = get_class_stat(targets);
    let mut ttc: f64 = 0.;
    for (_, count) in &target2count {
        ttc += count;
    }

    let mut target2percent: std::collections::HashMap<&String, f64> =
        std::collections::HashMap::new();
    for (t, count) in &target2count {
        let lcount: &f64 = count;
        let lttc: f64 = ttc;
        let percent: f64 = lcount / lttc;
        target2percent.insert(t, percent);
    }
    let mut new_map = std::collections::HashMap::new();
    count_dict.insert("tot".to_string(), new_map);
    add_class(&mut count_dict, &ts);
    add_class(&mut pmi_dict, &ts);
    for idx in 0..tokenized_texts.len() {
        let words = tokenized_texts[idx]
            .iter()
            .collect::<std::collections::HashSet<_>>();
        let target = targets[idx];

        for w in words {
            add_word(&mut count_dict, "tot", w);
            add_word(&mut count_dict, target, w);
        }
    }
    println!("Count dict {:?}", count_dict);
    for t in ts {
        let mut N_t: f64 = 0.;
        let mut N: f64 = 0.;
        for (tl, counts) in &count_dict {
            if tl == t {
                for (w, lcount) in counts {
                    N_t += lcount;
                }
            } else if tl == "tot" {
                for (w, lcount) in counts {
                    N += lcount;
                }
            }
        }
        for w in count_dict[&t.to_string()].keys() {
            let v = count_dict[&t.to_string()][w];
            if v > min_count as f64 {
                let wfreq = v / N_t + 10.0e-15;
                let tper = target2percent[&t.to_string()];
                let tword_count = count_dict[&"tot".to_string()][w];
                let pmi = -(wfreq / (tper * tword_count)).log2() / (wfreq).log2();
                let pmi_dict_t = pmi_dict.get_mut(&t.to_string()).unwrap();
                let count: &mut f64 = pmi_dict_t.entry(w.to_lowercase()).or_insert(0.);
                *count += pmi;
            }
        }
    }
    pmi_dict
}

fn get_doc_tfidf(
    words: &std::collections::HashSet<&&str>,
    word2text_count: &std::collections::HashMap<String, f64>,
    N: f64,
) -> std::collections::HashMap<String, f64> {
    let mut word2tfidf = std::collections::HashMap::new();
    let num_words = words.len();
    for word in words {
        let idf = (N / (word2text_count[&word.to_lowercase()])).log2();
        word2tfidf.insert(word.to_lowercase(), (1. / num_words as f64) * idf);
    }
    word2tfidf
}

fn classify_pmi_based(
    pmi_dict: &std::collections::HashMap<String, std::collections::HashMap<String, f64>>,
    word2text_count: &std::collections::HashMap<String, f64>,
    tokenized_test_texts: &Vec<Vec<&str>>,
    N: f64,
) -> Vec<usize> {
    let mut results = vec![0; tokenized_test_texts.len()];
    for idx in 0..tokenized_test_texts.len() {
        let words = tokenized_test_texts[idx]
            .iter()
            .collect::<std::collections::HashSet<_>>();
        let mut word2tfidf = get_doc_tfidf(&words, word2text_count, N);

        let mut tot_pmi = std::collections::HashMap::<&String, Vec<f64>>::default();
        let mut pmi = std::collections::HashMap::<&String, f64>::default();
        for k in pmi_dict {
            tot_pmi.insert(k.0, vec![]);
            for w in &words {
                // check if w is in pmi_dict[k]
                if pmi_dict[k.0].contains_key(&w.to_lowercase()) {
                    let pmi_w = pmi_dict[k.0][&w.to_lowercase()];
                    let tfidf_w = word2tfidf[&w.to_lowercase()];
                    let score = pmi_w * tfidf_w;
                    let mut tot_pmi_entry = tot_pmi.get_mut(k.0).unwrap();
                    tot_pmi_entry.push(score);
                }
            }
            let pmi_sum: &mut f64 = pmi.entry(k.0).or_insert(0.);
            // calculate sum of score for class k
            for score in tot_pmi[k.0].iter() {
                *pmi_sum += score;
            }
        }
        results[idx] = pmi
            .iter()
            .enumerate()
            .max_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap())
            .unwrap()
            .0;
    }
    results
}

mod tests {
    #[test]
    fn check_read_file(file_name: &str) {
        let text = read_file(file_name);
        assert_eq!(text.len(), 524);
    }
}

fn main() {
    // put file path from aruments
    let args: Vec<String> = std::env::args().collect();
    let file_name = &args[1];
    let filename = format!("{}docs.txt", file_name);
    let file_text = read_file(&filename);
    let lines = split_lines(&file_text);

    let tokenized_texts = tokenize_lines(lines);
    let word2text_count = get_word_stat(&tokenized_texts);

    let filename_labels = format!("{}labels.txt", file_name);
    let file_text_labels = read_file(&filename_labels);
    let lines_labels = split_lines(&file_text_labels);

    let target2count = get_class_stat(&lines_labels);

    let min_count = 0;

    let pmi_dict = create_pmi_dict(&tokenized_texts, &lines_labels, min_count);

    let results = classify_pmi_based(
        &pmi_dict,
        &word2text_count,
        &tokenized_texts,
        target2count.len() as f64,
    );
    println!("{:?}", results);
}
