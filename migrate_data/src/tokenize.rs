use gremlin_client::process::traversal::{GraphTraversalSource, SyncTerminator};
use ndarray::Array2;
use ndarray_linalg::norm::Norm;
use rust_stemmers::{Algorithm, Stemmer};
use std::{clone, collections::HashMap, fs::File, io::Read};
use whatlang::{detect, Lang};

use crate::models::VideosJson;

fn tokenize(text: &str) -> Vec<String> {
    let lang_info = detect(text);

    let stemmer_algorithm = match lang_info {
        Some(info) => match info.lang() {
            Lang::Eng => Algorithm::English,
            Lang::Fra => Algorithm::French,
            _ => Algorithm::English, // Default to English if language not supported
        },
        None => Algorithm::English, // Default to English if language detection fails
    };

    let stemmer = Stemmer::create(stemmer_algorithm);
    let tokens: Vec<String> = text
        .to_lowercase()
        .split_whitespace()
        .map(|word| stemmer.stem(word).to_string())
        .collect();
    tokens
}

fn compute_tf_matrix(videos: &[VideosJson]) -> (Array2<f64>, HashMap<String, usize>) {
    let mut word_to_idx = HashMap::new();
    let mut word_idx = 0;
    let mut tf_matrix: Vec<Vec<f64>> = Vec::new();

    for video in videos {
        let mut tf_vec = vec![0.0; word_to_idx.len()];
        let tokens =
            tokenize(&(video.title.to_lowercase() + " " + &video.description.to_lowercase())); // Tokenize is used here

        for token in tokens {
            if !word_to_idx.contains_key(&token) {
                word_to_idx.insert(token.clone(), word_idx);
                word_idx += 1;
                tf_vec.push(0.0);
            }
            let idx = *word_to_idx.get(&token).unwrap();
            tf_vec[idx] += 1.0;
        }
        tf_matrix.push(tf_vec);
    }
    for tf_vec in tf_matrix.iter_mut() {
        tf_vec.resize(word_to_idx.len(), 0.0);
    }

    println!("Dimensions: {:?}", (videos.len(), word_to_idx.len()));
    println!(
        "Flattened tf_matrix length: {:?}",
        tf_matrix.iter().flatten().count()
    );

    let array = Array2::from_shape_vec(
        (videos.len(), word_to_idx.len()),
        tf_matrix.into_iter().flatten().collect(),
    )
    .unwrap();
    (array, word_to_idx)
}

// fn compute_cosine_similarity(matrix: &Array2<f64>) -> Array2<f64> {
//     let row_norms = matrix.map_axis(ndarray::Axis(1), |row| row.norm_l2());
//     let normalized_matrix = matrix / &row_norms.insert_axis(ndarray::Axis(1));
//     let result = normalized_matrix.dot(&normalized_matrix.t());
//     panic!("Not implemented yet")
// }

fn compute_cosine_similarity(matrix: &Array2<f64>) -> Array2<f64> {
    let row_norms = matrix.map_axis(ndarray::Axis(1), |row| row.norm_l2());
    let normalized_matrix = matrix / &row_norms.insert_axis(ndarray::Axis(1));
    let mut similarity_matrix = Array2::eye(matrix.nrows());
    for i in 0..matrix.nrows() {
        for j in i + 1..matrix.nrows() {
            // let sim = normalized_matrix.row(i).dot(&normalized_matrix.row(j));
            let sim: f64 = normalized_matrix
                .row(i)
                .iter()
                .zip(normalized_matrix.row(j).iter())
                .map(|(a, b)| a * b)
                .sum();
            similarity_matrix[(i, j)] = sim;
            similarity_matrix[(j, i)] = sim;
        }
    }
    similarity_matrix
}

fn get_similar_items(
    video_id: &str,
    videos: &[VideosJson],
    similarity_matrix: &Array2<f64>,
    num_items: usize,
) -> Vec<VideosJson> {
    let index = videos
        .iter()
        .position(|video| video.video_id == video_id)
        .unwrap();
    let mut scores: Vec<(usize, f64)> = (0..similarity_matrix.nrows())
        .map(|i| (i, similarity_matrix[(index, i)]))
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Get the top-n most similar items, excluding the item itself (i.e., start from index 1)
    let top_items = scores
        .into_iter()
        .skip(1)
        .take(num_items)
        .map(|(index, _)| videos[index].clone())
        .collect();

    top_items
}

pub fn main() {
    // Get data from json file
    let mut file = File::open("../videos.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `Video`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let videos: Vec<VideosJson> = serde_json::from_str(&buff).expect("Failed to deserialize json");

    let (tf_matrix, _) = compute_tf_matrix(&videos);
    let cosine_sim = compute_cosine_similarity(&tf_matrix);

    let video_id = "9e5b5acf-da76-4b1c-89d9-e3ba1a1f71cc";
    let num_items = 2;
    let recommendations = get_similar_items(video_id, &videos, &cosine_sim, num_items);
    println!("{:#?}", recommendations);
}
fn add_similars_to_edges(
    g: &GraphTraversalSource<SyncTerminator>,
    videos: &[VideosJson],
    similarity_matrix: &Array2<f64>,
) {
    // the order of the videos in the similarity matrix is the same as the order of the videos in the videos array
    for (i, video_a) in videos.iter().enumerate() {
        let video_a_vertex = g
            .v(())
            .has(("video_id", video_a.video_id.to_owned()))
            .next()
            .unwrap();

        for (j, video_b) in videos.iter().enumerate() {
            if i != j {
                let video_b_vertex = g
                    .v(())
                    .has(("video_id", video_b.video_id.to_owned()))
                    .next()
                    .unwrap();

                if let (Some(video_a_vertex), Some(video_b_vertex)) =
                    (video_a_vertex.clone(), video_b_vertex)
                {
                    let similarity = similarity_matrix[(i, j)];
                    g.add_e("similar_to")
                        .from(&video_a_vertex)
                        .to(&video_b_vertex)
                        .property("similarity", similarity)
                        .next()
                        .unwrap();
                } else {
                    println!(
                        "Could not find vertex for video_a: {:?} or video_b: {:?}",
                        video_a, video_b
                    );
                }
            }
        }
    }
}

// fn add_similars_to_edges(ref g: &GraphTraversalSource<SyncTerminator>, videos: &[VideosJson], similarity_matrix: &Array2<f64>) {
//     // the order of the videos in the similarity matrix is the same as the order of the videos in the videos array
//     for video_a in videos {
//         let video_a_vertex = g.v(()).has(("video_id", video_a.video_id.to_owned())).next().unwrap();
//         for video_b in videos {
//             let video_b_vertex = g.v(()).has(("video_id", video_b.video_id.to_owned())).next().unwrap();

//             if let (Some(video_a_vertex), Some(video_b_vertex)) = (video_a_vertex.clone(), video_b_vertex) {
//                 let similarity = similarity_matrix[(videos.iter().position(|video| video.video_id == video_a.video_id).unwrap(), videos.iter().position(|video| video.video_id == video_b.video_id).unwrap())];
//                     g.add_e("similar_to")
//                         .from(&video_a_vertex)
//                         .to(&video_b_vertex)
//                         .property("similarity", similarity)
//                         .next()
//                         .unwrap();
//             } else {
//                 println!("Could not find vertex for video_a: {:?} or video_b: {:?}", video_a, video_b);
//             }
//         }
//     }
// }

pub fn recommendations(ref g: &GraphTraversalSource<SyncTerminator>) {
    // Get data from json file
    let mut file = File::open("../videos.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `Video`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let videos: Vec<VideosJson> = serde_json::from_str(&buff).expect("Failed to deserialize json");

    let (tf_matrix, _) = compute_tf_matrix(&videos);
    let cosine_sim = compute_cosine_similarity(&tf_matrix);

    add_similars_to_edges(&g, &videos, &cosine_sim);
}
