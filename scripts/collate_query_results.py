#!/usr/bin/env python

import sys
from pathlib import Path
from glob import glob

import rdflib
from rdflib import Graph
import progressbar

def load_all_rdf(input_directory):
    all_rdf_paths = list(input_directory.glob("**/*.rdf"))

    progress_bar = progressbar.ProgressBar(max_value=len(all_rdf_paths))
    graph = Graph()
    try:
        for rdf_path in progress_bar(all_rdf_paths):
            graph.parse(str(rdf_path))
    except:
        pass

    return graph

def main():
    argc = len(sys.argv)
    if argc != 4:
        raise ValueError(f"Illegal number of arguments passed, {argc}")

    input_directory = Path.cwd() / Path(sys.argv[1])
    output_path = Path.cwd() / Path(sys.argv[2])
    query_path = Path.cwd() / Path(sys.argv[3])

    if not input_directory.exists():
        raise FileNotFoundError("Input catalog directory does not exist")

    graph = load_all_rdf(input_directory)

    with open(query_path, 'r') as query_file:
        query_text = query_file.read();
        query_result = graph.query(query_text)
        query_result.serialize(str(output_path), format='csv', encoding='utf-8')

if __name__ == '__main__':
    try:
        main()
    except (ValueError, FileNotFoundError) as e:
        print(e, file=sys.stderr)
        print("usage: ./collate_query_results.py <catalog-directory> <output-path> <query-path>", file=sys.stderr)