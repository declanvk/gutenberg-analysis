#!/usr/bin/env python

import sys
from pathlib import Path
from glob import glob

import rdflib
from rdflib import Graph
import progressbar

def load_all_rdf(input_directory):
    all_rdf_paths = list(input_directory.glob("**/*.rdf"))

    graph = Graph()

    progress_bar = progressbar.ProgressBar(max_value=len(all_rdf_paths))

    for rdf_path in progress_bar(all_rdf_paths):
        graph.parse(str(rdf_path))

    return graph

def main():
    argc = len(sys.argv)
    if argc != 3:
        raise ValueError(f"Illegal number of arguments passed, {argc}")

    input_directory = Path.cwd() / Path(sys.argv[1])
    output_path = Path.cwd() / Path(sys.argv[2])

    if not input_directory.exists():
        raise FileNotFoundError("Input catalog directory does not exist")

    graph = load_all_rdf(input_directory)
    graph.serialize(str(output_path))

if __name__ == '__main__':
    try:
        main()
    except (ValueError, FileNotFoundError) as e:
        print(e, file=sys.stderr)
        print("usage: ./compact_catalog.py <catalog-directory> <output-path>", file=sys.stderr)